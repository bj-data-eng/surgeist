//! Iterative typed numerical lowering into the existing canonical evaluator.

use super::super::{
    expression::{
        ExpressionDescriptor, ExpressionRecord, Number, NumericLeaf, OperandReference, Variable,
    },
    literal::{LiteralAction, LiteralConversion},
    protocol::{
        ColorProtocolError, CssColorBank, CssColorOperationOwner, CssColorSlot, Data, RecordTable,
    },
};
use super::CssColorPathStep;
use super::input::Operand;
use super::position::Position;
use super::site::{SiteRecord, TraversalRecord};
use super::spelling::{self, NameSpelling, NumberSpelling};
use crate::numeric::{bounded_expression as engine, color_operands as tree};
use crate::{
    CssComponentValueRef, CssNumericLiteralRef, CssNumericType, CssValueOrigin, CssValueTokenRef,
};

#[derive(Clone, Copy)]
pub(crate) struct Frame<'css> {
    node: tree::Node<'css>,
    header: tree::Header<'css>,
    site: usize,
    closed: bool,
    parent: Option<usize>,
    depth: usize,
    next: usize,
    last: Option<usize>,
    syntax_next: usize,
    strategy_origin: Option<&'css CssValueOrigin>,
    operator: tree::Operator,
    operator_origin: Option<&'css CssValueOrigin>,
    child: Option<tree::Node<'css>>,
    child_read: bool,
    operator_site: Option<usize>,
}
impl std::fmt::Debug for Frame<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthoredNumericFrame")
            .field("node", &self.node)
            .field("site", &self.site)
            .field("next", &self.next)
            .finish_non_exhaustive()
    }
}
impl Frame<'_> {
    fn children(self) -> usize {
        match self.header.kind {
            tree::Kind::Group => 1,
            tree::Kind::Sum { len }
            | tree::Kind::Product { len }
            | tree::Kind::Function { len, .. } => len,
            _ => 0,
        }
    }
    fn identical(self, other: Self) -> bool {
        tree::same(self.node, other.node)
            && self.site == other.site
            && self.parent == other.parent
            && self.closed == other.closed
            && self.depth == other.depth
            && self.next == other.next
            && self.last == other.last
            && self.child_read == other.child_read
            && self.operator_site == other.operator_site
            && self.syntax_next == other.syntax_next
            && same_operator(self.operator, other.operator)
            && same_origin(self.strategy_origin, other.strategy_origin)
            && same_origin(self.operator_origin, other.operator_origin)
            && match (self.child, other.child) {
                (Some(a), Some(b)) => tree::same(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}
fn same_origin(a: Option<&CssValueOrigin>, b: Option<&CssValueOrigin>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => std::ptr::eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
fn same_operator(a: tree::Operator, b: tree::Operator) -> bool {
    match (a, b) {
        (tree::Operator::None, tree::Operator::None) => true,
        (tree::Operator::Sum(a), tree::Operator::Sum(b)) => a == b,
        (tree::Operator::Product(a), tree::Operator::Product(b)) => a == b,
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Action {
    ReadHeader,
    ReadChild { ordinal: usize },
    ReadSyntax { index: usize },
    ReadLiteral,
    ReadNameHeader,
    ReadUnit { bytes: usize },
    ReadByte { offset: usize },
    Read { bank: CssColorBank, index: usize },
    Append { bank: CssColorBank, index: usize },
    Replace { bank: CssColorBank, index: usize },
    Compute,
    Done,
}
#[derive(Clone, Copy)]
enum Commit<'css> {
    Site { frame: Frame<'css> },
    Enter(Frame<'css>),
    OperatorSite(Frame<'css>),
    Leaf { frame: Frame<'css>, leaf: usize },
    Node { frame: Frame<'css>, root: usize },
    Suspend { frame: Frame<'css>, index: usize },
    Return { frame: Frame<'css>, root: usize },
    Link { frame: Frame<'css>, root: usize },
}
#[expect(
    clippy::large_enum_variant,
    reason = "Only one typed lowering or exact literal conversion is active; fixed registers are admitted inline without a hidden allocation"
)]
enum State<'css> {
    Header {
        node: tree::Node<'css>,
        parent: Option<usize>,
        parent_site: usize,
        depth: usize,
        edge: Option<CssColorPathStep>,
    },
    Frame(Frame<'css>),
    Child(Frame<'css>),
    Syntax(Frame<'css>),
    Literal {
        frame: Frame<'css>,
        literal: CssNumericLiteralRef<'css>,
    },
    Unit {
        frame: Frame<'css>,
        literal: CssNumericLiteralRef<'css>,
        unit: Option<&'css str>,
    },
    Number {
        frame: Frame<'css>,
        writer: NumberSpelling<'css>,
    },
    ProfileName {
        frame: Frame<'css>,
        name: &'css crate::CssColorProfileComponentName,
    },
    Name {
        frame: Frame<'css>,
        writer: NameSpelling<'css>,
    },
    ReadNumber {
        frame: Frame<'css>,
        record: usize,
    },
    Convert {
        frame: Frame<'css>,
        record: usize,
        cursor: LiteralConversion<'css>,
    },
    Pending {
        data: Data<'css>,
        bank: CssColorBank,
        commit: Commit<'css>,
    },
    Parent {
        index: usize,
        child: tree::Node<'css>,
        root: usize,
    },
    Done {
        root: usize,
        ty: CssNumericType,
        site: usize,
        closed: bool,
    },
}
pub(super) struct NumericLowering<'css> {
    owner: &'css CssColorOperationOwner,
    phase: usize,
    state: State<'css>,
    algorithm_end: usize,
    arithmetic_start: usize,
    arithmetic_end: usize,
    position: Position,
}
impl<'css> NumericLowering<'css> {
    pub(super) fn start(
        owner: &'css CssColorOperationOwner,
        phase: usize,
        node: tree::Node<'css>,
        parent_site: usize,
        depth: usize,
        algorithm_end: usize,
        arithmetic_end: usize,
    ) -> Self {
        Self {
            owner,
            phase,
            state: State::Header {
                node,
                parent: None,
                parent_site,
                depth,
                edge: None,
            },
            algorithm_end,
            arithmetic_start: arithmetic_end,
            arithmetic_end,
            position: Position {
                site: algorithm_end,
                depth,
                edge: None,
            },
        }
    }
    pub(super) fn action(&self) -> Action {
        match &self.state {
            State::Header { .. } => Action::ReadHeader,
            State::Child(frame) => Action::ReadChild {
                ordinal: frame.next,
            },
            State::Syntax(frame) => Action::ReadSyntax {
                index: frame.syntax_next,
            },
            State::Literal { .. } => Action::ReadLiteral,
            State::ProfileName { .. } => Action::ReadNameHeader,
            State::Unit { unit, .. } => Action::ReadUnit {
                bytes: unit.map_or(0, str::len),
            },
            State::Number { writer, .. } => spelling_action(writer.action()),
            State::Name { writer, .. } => spelling_action(writer.action()),
            State::ReadNumber { record, .. } => Action::Read {
                bank: CssColorBank::Algorithm,
                index: *record,
            },
            State::Convert { cursor, .. } => match cursor.action() {
                LiteralAction::ReadSource { index }
                | LiteralAction::ReadScaled { index }
                | LiteralAction::ReadScratch { index }
                | LiteralAction::ReadHeader { index } => Action::Read {
                    bank: CssColorBank::Algorithm,
                    index,
                },
                LiteralAction::Append { index } => Action::Append {
                    bank: CssColorBank::Algorithm,
                    index,
                },
                LiteralAction::Replace { index } => Action::Replace {
                    bank: CssColorBank::Algorithm,
                    index,
                },
                _ => Action::Compute,
            },
            State::Pending { bank, .. } => Action::Append {
                bank: *bank,
                index: match bank {
                    CssColorBank::Algorithm => self.algorithm_end,
                    CssColorBank::Arithmetic => self.arithmetic_end,
                    CssColorBank::Evaluation => unreachable!("lowering owns no evaluation records"),
                },
            },
            State::Parent { index, .. } => Action::Read {
                bank: CssColorBank::Algorithm,
                index: *index,
            },
            State::Done { .. } => Action::Done,
            State::Frame(_) => Action::Compute,
        }
    }
    pub(super) fn input_bytes(&self) -> Result<usize, ColorProtocolError> {
        Ok(match &self.state {
            State::Header { .. } => tree::header_bytes(),
            State::Child(frame) => tree::child_bytes(frame.header.kind),
            State::Syntax(_) => tree::syntax_item_bytes(),
            State::Literal { .. } => std::mem::size_of::<crate::CssComponentValue>(),
            State::ProfileName { .. } => std::mem::size_of::<crate::CssColorProfileComponentName>(),
            State::Unit { unit, .. } => std::mem::size_of::<crate::CssComponentValue>()
                .checked_add(unit.map_or(0, str::len))
                .ok_or(ColorProtocolError::Overflow)?,
            State::Number { .. } | State::Name { .. }
                if matches!(self.action(), Action::ReadByte { .. }) =>
            {
                1
            }
            _ => 0,
        })
    }
    fn pending(
        &mut self,
        data: Data<'css>,
        bank: CssColorBank,
        commit: Commit<'css>,
    ) -> Result<(), ColorProtocolError> {
        let end = if bank == CssColorBank::Algorithm {
            self.algorithm_end
        } else {
            self.arithmetic_end
        };
        end.checked_add(1).ok_or(ColorProtocolError::Overflow)?;
        self.state = State::Pending { data, bank, commit };
        Ok(())
    }
    pub(super) fn read_authored(&mut self) -> Result<(), ColorProtocolError> {
        match &mut self.state {
            State::Header {
                node,
                parent,
                parent_site,
                depth,
                edge,
            } => {
                let node = *node;
                let parent = *parent;
                let parent_site = *parent_site;
                let depth = *depth;
                let edge = *edge;
                let header = tree::header(node);
                let closed = !header
                    .closing
                    .is_some_and(|origin| matches!(origin, CssValueOrigin::ImplicitClosure { .. }));
                let origin = header.origin;
                let site = self.algorithm_end;
                let frame = Frame {
                    node,
                    header,
                    site,
                    closed,
                    parent,
                    depth,
                    next: 0,
                    last: None,
                    syntax_next: 0,
                    strategy_origin: None,
                    operator: tree::Operator::None,
                    operator_origin: None,
                    child: None,
                    child_read: false,
                    operator_site: None,
                };
                self.pending(
                    Data::AuthoredSite(SiteRecord {
                        operand: Operand::Expression(node),
                        parent: Some(parent_site),
                        edge,
                        depth,
                        origin: Some(origin),
                    }),
                    CssColorBank::Algorithm,
                    Commit::Site { frame },
                )
            }
            State::Child(frame) => {
                let mut frame = *frame;
                let child =
                    tree::child(frame.node, frame.next).ok_or(ColorProtocolError::WrongIndex)?;
                frame.child = child.expression;
                frame.operator = child.operator;
                frame.operator_origin = None;
                frame.operator_site = None;
                frame.child_read = true;
                self.state = if matches!(frame.operator, tree::Operator::None) {
                    State::Frame(frame)
                } else {
                    State::Syntax(frame)
                };
                Ok(())
            }
            State::Syntax(frame) => {
                let mut frame = *frame;
                if frame.syntax_next >= frame.header.syntax_len {
                    return Err(ColorProtocolError::WrongIndex);
                }
                let item = tree::syntax_item(frame.node, frame.syntax_next)
                    .ok_or(ColorProtocolError::WrongIndex)?;
                frame.syntax_next = frame
                    .syntax_next
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                if matches!(
                    frame.header.kind,
                    tree::Kind::Function {
                        strategy: Some(_),
                        ..
                    }
                ) && frame.strategy_origin.is_none()
                {
                    // The checked numeric owner already selected the strategy.
                    // This one-pass observation retains its original token origin
                    // without reading/classifying its decoded name again.
                    match item.view() {
                        CssComponentValueRef::Token(CssValueTokenRef::Ident(_)) => {
                            frame.strategy_origin = Some(item.origin());
                            self.state = State::Frame(frame);
                        }
                        CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                        | CssComponentValueRef::Comment(_) => self.state = State::Syntax(frame),
                        _ => return Err(ColorProtocolError::WrongSlot),
                    }
                    return Ok(());
                }
                let matched = matches!(
                    (frame.operator, item.view()),
                    (
                        tree::Operator::Sum(crate::CssCalculationSumOperator::Add),
                        CssComponentValueRef::Token(CssValueTokenRef::Delim('+')),
                    ) | (
                        tree::Operator::Sum(crate::CssCalculationSumOperator::Subtract),
                        CssComponentValueRef::Token(CssValueTokenRef::Delim('-')),
                    ) | (
                        tree::Operator::Product(crate::CssCalculationProductOperator::Multiply),
                        CssComponentValueRef::Token(CssValueTokenRef::Delim('*')),
                    ) | (
                        tree::Operator::Product(crate::CssCalculationProductOperator::Divide),
                        CssComponentValueRef::Token(CssValueTokenRef::Delim('/')),
                    )
                );
                if matched {
                    frame.operator_origin = Some(item.origin());
                    frame.operator_site = Some(self.algorithm_end);
                    self.pending(
                        Data::AuthoredSite(SiteRecord {
                            operand: Operand::Expression(frame.node),
                            parent: Some(frame.site),
                            edge: None,
                            depth: frame.depth,
                            origin: Some(item.origin()),
                        }),
                        CssColorBank::Algorithm,
                        Commit::OperatorSite(frame),
                    )
                } else {
                    self.state = State::Syntax(frame);
                    Ok(())
                }
            }
            State::Literal { frame, literal } => {
                let frame = *frame;
                let literal = *literal;
                let unit = literal.unit();
                // This action observes the actual original ComponentValue record.
                let _ = literal.numeric();
                let _ = literal.origin();
                self.state = State::Unit {
                    frame,
                    literal,
                    unit,
                };
                Ok(())
            }
            State::Unit { frame, literal, .. } => {
                let frame = *frame;
                let literal = *literal;
                let unit = literal.canonical_unit();
                let percentage = literal
                    .numeric_type()
                    .is(crate::CssNumericDimension::Percentage);
                self.state = State::Number {
                    frame,
                    writer: NumberSpelling::start(
                        self.owner,
                        self.phase,
                        literal.numeric(),
                        unit,
                        percentage,
                        frame.site,
                        self.algorithm_end,
                    ),
                };
                Ok(())
            }
            State::ProfileName { frame, name } => {
                let frame = *frame;
                let name = *name;
                self.state = State::Name {
                    frame,
                    writer: NameSpelling::start(
                        self.owner,
                        self.phase,
                        name.as_str(),
                        frame.site,
                        self.algorithm_end,
                    ),
                };
                Ok(())
            }
            State::Number { writer, .. } => writer.read_byte(),
            State::Name { writer, .. } => writer.read_byte(),
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    pub(super) fn read(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'css>,
    ) -> Result<(), ColorProtocolError> {
        if self.action() != (Action::Read { bank, index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        slot.qualify(self.owner, self.phase, index, bank)?;
        match &mut self.state {
            State::ReadNumber { frame, record } => {
                let frame = *frame;
                let record = *record;
                let table = RecordTable {
                    owner: self.owner,
                    phase: self.phase,
                    length: self.algorithm_end,
                };
                let cursor =
                    LiteralConversion::start_authored(&table, record, slot, self.algorithm_end)?;
                self.state = State::Convert {
                    frame,
                    record,
                    cursor,
                };
                Ok(())
            }
            State::Convert { cursor, .. } => cursor.read(index, slot),
            State::Parent {
                index: expected,
                child,
                root,
            } => {
                if index != *expected {
                    return Err(ColorProtocolError::WrongIndex);
                }
                let Data::AuthoredNumericFrame(frame) = slot.data else {
                    return Err(ColorProtocolError::WrongSlot);
                };
                if !frame
                    .child
                    .is_some_and(|expected| tree::same(expected, *child))
                {
                    return Err(ColorProtocolError::WrongSlot);
                }
                let root = *root;
                self.position = Position {
                    site: frame.site,
                    depth: frame.depth,
                    edge: None,
                };
                self.pending(
                    Data::AuthoredTraversal(TraversalRecord::Return { site: frame.site }),
                    CssColorBank::Arithmetic,
                    Commit::Return { frame, root },
                )
            }
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    pub(super) fn record(
        &self,
        bank: CssColorBank,
        index: usize,
    ) -> Result<CssColorSlot<'css>, ColorProtocolError> {
        if !matches!(self.action(),Action::Append {bank:b,index:i}|Action::Replace {bank:b,index:i} if b==bank && i==index)
        {
            return Err(ColorProtocolError::WrongAction);
        }
        match &self.state {
            State::Number { writer, .. } => writer.record(index),
            State::Name { writer, .. } => writer.record(index),
            State::Convert { cursor, .. } => cursor.record(index),
            State::Pending { data, .. } => {
                Ok(CssColorSlot::new(self.owner, self.phase, index, *data))
            }
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    pub(super) fn written(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        let action = self.action();
        if !matches!(action,Action::Append {bank:b,index:i}|Action::Replace {bank:b,index:i} if b==bank && i==index)
        {
            return Err(ColorProtocolError::WrongAction);
        }
        slot.qualify(self.owner, self.phase, index, bank)?;
        match &mut self.state {
            State::Number { writer, .. } => {
                writer.written(index, slot)?;
                self.algorithm_end = writer.algorithm_end();
                return Ok(());
            }
            State::Name { writer, .. } => {
                writer.written(index, slot)?;
                self.algorithm_end = writer.algorithm_end();
                return Ok(());
            }
            State::Convert { cursor, .. } => {
                cursor.written(index, slot)?;
                self.algorithm_end = cursor.scratch_end();
                return Ok(());
            }
            _ => {}
        }
        let State::Pending { data, commit, .. } = self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        if !same_data(data, slot.data) {
            return Err(ColorProtocolError::WrongSlot);
        }
        if bank == CssColorBank::Algorithm {
            self.algorithm_end = self
                .algorithm_end
                .checked_add(1)
                .ok_or(ColorProtocolError::Overflow)?;
        } else {
            self.arithmetic_end = self
                .arithmetic_end
                .checked_add(1)
                .ok_or(ColorProtocolError::Overflow)?;
        }
        match commit {
            Commit::Site { frame } => self.pending(
                Data::AuthoredTraversal(TraversalRecord::Enter { site: frame.site }),
                CssColorBank::Arithmetic,
                Commit::Enter(frame),
            )?,
            Commit::OperatorSite(frame) => self.state = State::Frame(frame),
            Commit::Enter(frame) => {
                self.state = match frame.header.kind {
                    tree::Kind::Value(literal) => State::Literal { frame, literal },
                    _ => State::Frame(frame),
                };
            }
            Commit::Leaf { frame, leaf } => self.pending(
                Data::Expression(ExpressionRecord::authored(engine::authored::node(
                    engine::NodeKind::Leaf { leaf },
                    frame.header.ty,
                    frame.site,
                ))),
                CssColorBank::Arithmetic,
                Commit::Node {
                    frame,
                    root: self.arithmetic_end,
                },
            )?,
            Commit::Node { frame, root } => {
                self.state = if let Some(index) = frame.parent {
                    State::Parent {
                        index,
                        child: frame.node,
                        root,
                    }
                } else {
                    State::Done {
                        root,
                        ty: frame.header.ty,
                        site: frame.site,
                        closed: frame.closed,
                    }
                }
            }
            Commit::Suspend { frame, index } => {
                let child = frame.child.ok_or(ColorProtocolError::WrongSlot)?;
                let depth = frame
                    .depth
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                let edge = Some(CssColorPathStep::ExpressionOperand(frame.next));
                self.position = Position {
                    site: self.algorithm_end,
                    depth,
                    edge,
                };
                self.state = State::Header {
                    node: child,
                    parent: Some(index),
                    parent_site: frame.site,
                    depth,
                    edge,
                };
            }
            Commit::Return { frame, root } => self.link(frame, Some(root))?,
            Commit::Link { mut frame, root } => {
                frame.last = Some(root);
                frame.next = frame
                    .next
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                frame.child = None;
                frame.child_read = false;
                self.state = State::Frame(frame);
            }
        }
        Ok(())
    }
    fn leaf(&mut self, frame: Frame<'css>, value: NumericLeaf) -> Result<(), ColorProtocolError> {
        self.pending(
            Data::NumericLeaf(value),
            CssColorBank::Algorithm,
            Commit::Leaf {
                frame,
                leaf: self.algorithm_end,
            },
        )
    }
    fn link(&mut self, frame: Frame<'css>, child: Option<usize>) -> Result<(), ColorProtocolError> {
        let root = self.arithmetic_end;
        let record = match frame.header.kind {
            tree::Kind::Group => {
                let mut frame = frame;
                frame.last = child;
                frame.next = 1;
                frame.child = None;
                frame.child_read = false;
                self.state = State::Frame(frame);
                return Ok(());
            }
            tree::Kind::Sum { .. } => engine::authored::term(
                frame.last,
                frame.next,
                child.ok_or(ColorProtocolError::WrongSlot)?,
                match frame.operator {
                    tree::Operator::Sum(op) => Some(op),
                    tree::Operator::None => None,
                    _ => return Err(ColorProtocolError::WrongSlot),
                },
                frame.operator_site,
            ),
            tree::Kind::Product { .. } => engine::authored::factor(
                frame.last,
                frame.next,
                child.ok_or(ColorProtocolError::WrongSlot)?,
                match frame.operator {
                    tree::Operator::Product(op) => Some(op),
                    tree::Operator::None => None,
                    _ => return Err(ColorProtocolError::WrongSlot),
                },
                frame.operator_site,
            ),
            tree::Kind::Function { .. } => {
                engine::authored::argument(frame.last, frame.next, child)
            }
            _ => return Err(ColorProtocolError::WrongSlot),
        };
        self.pending(
            Data::Expression(ExpressionRecord::authored(record)),
            CssColorBank::Arithmetic,
            Commit::Link { frame, root },
        )
    }
    pub(super) fn kernel(&self) -> crate::color::CssColorKernel {
        match &self.state {
            State::Convert { cursor, .. } => cursor.kernel(),
            _ => crate::color::CssColorKernel::Transition,
        }
    }
    pub(super) fn compute(&mut self) -> Result<(), ColorProtocolError> {
        if self.action() != Action::Compute {
            return Err(ColorProtocolError::WrongAction);
        }
        match &mut self.state {
            State::Number { frame, writer } => {
                if let spelling::Action::Done { record } = writer.action() {
                    let frame = *frame;
                    // Header is read as a separate bank observation by conversion.
                    self.state = State::ReadNumber { frame, record };
                } else {
                    writer.compute()?;
                }
                Ok(())
            }
            State::Name { frame, writer } => {
                if let spelling::Action::Done { record } = writer.action() {
                    let frame = *frame;
                    let units = super::super::protocol::RecordRange {
                        start: writer.units_start(),
                        end: record,
                    };
                    self.leaf(
                        frame,
                        NumericLeaf::Variable {
                            token: OperandReference::Authored {
                                index: record,
                                site: frame.site,
                            },
                            binding: Variable::Profile { name: units },
                        },
                    )
                } else {
                    writer.compute()
                }
            }
            State::Convert {
                frame,
                record,
                cursor,
            } => {
                if let LiteralAction::Done(value) = cursor.action() {
                    let frame = *frame;
                    let record = *record;
                    let leaf = super::super::expression::converted_literal(
                        value,
                        OperandReference::Authored {
                            index: record,
                            site: frame.site,
                        },
                        frame.header.ty,
                    );
                    self.leaf(frame, leaf)
                } else {
                    cursor.compute()
                }
            }
            State::Frame(frame) => {
                let frame = *frame;
                if matches!(
                    frame.header.kind,
                    tree::Kind::Function {
                        strategy: Some(_),
                        ..
                    }
                ) && frame.strategy_origin.is_none()
                {
                    self.state = State::Syntax(frame);
                    return Ok(());
                }
                match frame.header.kind {
                    tree::Kind::Constant(value) => self.leaf(
                        frame,
                        NumericLeaf::Literal(Number {
                            value: value.magnitude(),
                            ty: frame.header.ty,
                        }),
                    ),
                    tree::Kind::Variable(channel) => self.leaf(
                        frame,
                        NumericLeaf::Variable {
                            token: OperandReference::Authored {
                                index: frame.site,
                                site: frame.site,
                            },
                            binding: Variable::Channel(channel),
                        },
                    ),
                    tree::Kind::ProfileChannel(name) => {
                        self.state = State::ProfileName { frame, name };
                        Ok(())
                    }
                    tree::Kind::Size | tree::Kind::TreeCounting => {
                        Err(ColorProtocolError::WrongSlot)
                    }
                    tree::Kind::Value(_) => Err(ColorProtocolError::WrongAction),
                    _ if frame.next == frame.children() => {
                        let last = frame.last.ok_or(ColorProtocolError::WrongSlot)?;
                        let kind = match frame.header.kind {
                            tree::Kind::Group => engine::NodeKind::Group { child: last },
                            tree::Kind::Sum { len } => engine::NodeKind::Sum { last, count: len },
                            tree::Kind::Product { len } => {
                                engine::NodeKind::Product { last, count: len }
                            }
                            tree::Kind::Function {
                                function,
                                len,
                                strategy,
                            } => engine::NodeKind::Function {
                                function,
                                last,
                                count: len,
                                strategy,
                            },
                            _ => unreachable!("aggregate kind"),
                        };
                        self.pending(
                            Data::Expression(ExpressionRecord::authored(engine::authored::node(
                                kind,
                                frame.header.ty,
                                frame.site,
                            ))),
                            CssColorBank::Arithmetic,
                            Commit::Node {
                                frame,
                                root: self.arithmetic_end,
                            },
                        )
                    }
                    _ if frame.child.is_some() => self.pending(
                        Data::AuthoredNumericFrame(frame),
                        CssColorBank::Algorithm,
                        Commit::Suspend {
                            frame,
                            index: self.algorithm_end,
                        },
                    ),
                    _ if matches!(frame.header.kind, tree::Kind::Function { .. })
                        && frame.child_read =>
                    {
                        self.link(frame, None)
                    }
                    _ => {
                        self.state = State::Child(frame);
                        Ok(())
                    }
                }
            }
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    pub(super) fn finish(&self) -> Result<ExpressionDescriptor, ColorProtocolError> {
        let State::Done {
            root,
            ty,
            site,
            closed,
        } = self.state
        else {
            return Err(ColorProtocolError::Terminal);
        };
        ExpressionDescriptor::authored(
            self.arithmetic_start,
            self.arithmetic_end,
            root,
            ty,
            site,
            closed,
        )
    }
    pub(super) const fn position(&self) -> Position {
        self.position
    }
    pub(super) const fn algorithm_end(&self) -> usize {
        self.algorithm_end
    }
    pub(super) const fn arithmetic_end(&self) -> usize {
        self.arithmetic_end
    }
}
fn spelling_action(action: spelling::Action) -> Action {
    match action {
        spelling::Action::ReadByte { offset } => Action::ReadByte { offset },
        spelling::Action::Compute | spelling::Action::Done { .. } => Action::Compute,
        spelling::Action::Append { index } => Action::Append {
            bank: CssColorBank::Algorithm,
            index,
        },
    }
}
fn same_data(left: Data<'_>, right: Data<'_>) -> bool {
    match (left, right) {
        (Data::AuthoredSite(a), Data::AuthoredSite(b)) => a.identical(b),
        (Data::AuthoredNumericFrame(a), Data::AuthoredNumericFrame(b)) => a.identical(b),
        (Data::AuthoredTraversal(a), Data::AuthoredTraversal(b)) => a == b,
        (Data::Expression(a), Data::Expression(b)) => a == b,
        (Data::NumericLeaf(a), Data::NumericLeaf(b)) => a.same(b),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::expression::{EvaluationAction, ExpressionTable, NumericEvaluation};
    use crate::color::grammar::GrammarInput;

    /// The `<color>` the production `color` property parser admits for `source`.
    fn production_color(source: &str) -> crate::CssColor {
        let report = crate::parse_property_value_text(
            source,
            crate::CssPropertyNameRef::Known(crate::CssKnownProperty::Color),
            crate::CssImportance::Normal,
        );
        // CSS Syntax 3 (CRD 2026-10-01) §5.5.10 closes an unclosed function at
        // EOF as a parse error that retains the value.
        assert!(
            report
                .diagnostics()
                .iter()
                .all(|diagnostic| diagnostic.action()
                    == crate::CssRecoveryAction::RetainWithImplicitClosure),
            "{source}: {:?}",
            report.diagnostics()
        );
        let declaration = report.into_parts().0.expect("retained color value");
        let Some(crate::CssKnownPropertyValueRef::Color(value)) =
            declaration.known().and_then(|known| known.property_value())
        else {
            panic!("typed color: {declaration:?}");
        };
        value.value().clone()
    }

    #[test]
    fn typed_children_lower_into_the_single_postorder_evaluator_with_original_optional_arguments() {
        for (source, expected) in [
            ("calc(2 + 3 * 4)", 14.0_f64),
            ("min(4, 2, 8)", 2.0),
            ("clamp(none, 3, 2)", 2.0),
            ("round(up, 5.1, 2)", 6.0),
            ("calc(-0)", 0.0),
            ("sqrt(4)", 2.0),
            ("sin(100grad)", 1.0),
            ("calc(1e-9999)", 0.0),
            ("rgb(0 0 calc(2 + 3", 5.0),
        ] {
            let recovered = if source.starts_with("rgb(") {
                Some(production_color(source))
            } else {
                None
            };
            let calculation = if recovered.is_none() {
                Some(
                    crate::CssNumberCalculation::try_from_components(
                        crate::parse_component_values(source).unwrap(),
                    )
                    .unwrap(),
                )
            } else {
                None
            };
            let node = if let Some(color) = &recovered {
                let crate::CssColorComponent::NumberCalculation(value) =
                    &color.rgb_value().unwrap().channels()[2]
                else {
                    panic!("original recovered calculation")
                };
                assert!(!color.is_closed());
                tree::node(&value.expression)
            } else {
                tree::node(&calculation.as_ref().unwrap().expression)
            };
            if recovered.is_some() {
                let Some(CssValueOrigin::ImplicitClosure { opening, at }) =
                    tree::header(node).closing
                else {
                    panic!("original implicit closing origin")
                };
                assert_eq!(at.source().as_str(), source);
                assert!(opening.source().same_snapshot(at.source()));
                assert_eq!(at.span().start().byte_offset().value(), source.len());
            }
            let owner = CssColorOperationOwner::new();
            let phase = owner.current_phase();
            let next = owner.prepare_advance(phase).unwrap();
            owner.commit_phase(phase, next);
            let mut algorithm = vec![CssColorSlot::new(
                &owner,
                next,
                0,
                Data::AuthoredSite(SiteRecord {
                    operand: Operand::Expression(node),
                    parent: None,
                    edge: None,
                    depth: 0,
                    origin: None,
                }),
            )];
            let mut arithmetic = Vec::new();
            let mut lower = NumericLowering::start(&owner, next, node, 0, 0, algorithm.len(), 0);
            let mut done = false;
            for _ in 0..10_000_000 {
                match lower.action() {
                    Action::ReadHeader
                    | Action::ReadChild { .. }
                    | Action::ReadSyntax { .. }
                    | Action::ReadLiteral
                    | Action::ReadNameHeader
                    | Action::ReadUnit { .. }
                    | Action::ReadByte { .. } => lower.read_authored().unwrap(),
                    Action::Read { bank, index } => {
                        let slots = match bank {
                            CssColorBank::Algorithm => &algorithm,
                            CssColorBank::Arithmetic => &arithmetic,
                            CssColorBank::Evaluation => panic!("no evaluation during lowering"),
                        };
                        lower.read(bank, index, &slots[index]).unwrap();
                    }
                    Action::Append { bank, index } | Action::Replace { bank, index } => {
                        let replacing = matches!(lower.action(), Action::Replace { .. });
                        let slots = match bank {
                            CssColorBank::Algorithm => &mut algorithm,
                            CssColorBank::Arithmetic => &mut arithmetic,
                            CssColorBank::Evaluation => panic!("no evaluation during lowering"),
                        };
                        let record = lower.record(bank, index).unwrap();
                        if replacing {
                            slots[index] = record;
                        } else {
                            assert_eq!(index, slots.len());
                            slots.push(record);
                        }
                        lower.written(bank, index, &slots[index]).unwrap();
                    }
                    Action::Compute => lower.compute().unwrap(),
                    Action::Done => {
                        done = true;
                        break;
                    }
                }
            }
            assert!(done, "lowering did not finish: {source}");
            assert_eq!(lower.algorithm_end(), algorithm.len());
            assert_eq!(lower.arithmetic_end(), arithmetic.len());
            let input = GrammarInput::Authored(RecordTable {
                owner: &owner,
                phase: next,
                length: algorithm.len(),
            });
            let table = ExpressionTable::from_descriptor(&input, lower.finish().unwrap()).unwrap();
            let mut evaluation = NumericEvaluation::start(&table, 0).unwrap();
            let mut values = Vec::new();
            let mut actual = None;
            let mut visits = 0;
            for _ in 0..10_000_000 {
                match evaluation.action() {
                    EvaluationAction::ReadExpression { index } => {
                        evaluation
                            .read_expression(index, &arithmetic[index])
                            .unwrap();
                        if let Data::AuthoredTraversal(event) = arithmetic[index].data {
                            let Data::AuthoredSite(site) = algorithm[event.site()].data else {
                                panic!("actual site record")
                            };
                            assert!(matches!(site.operand, Operand::Expression(_)));
                            assert_eq!(evaluation.authored_site(), Some(event.site()));
                            visits += 1;
                        }
                    }
                    EvaluationAction::ReadLeaf { index } => {
                        evaluation.read_leaf(index, &algorithm[index]).unwrap()
                    }
                    EvaluationAction::ReadPrevious { index }
                    | EvaluationAction::ReadOperand { index } => {
                        evaluation.read_value(index, &values[index]).unwrap()
                    }
                    EvaluationAction::Append { index } => {
                        assert_eq!(index, values.len());
                        values.push(evaluation.record(index).unwrap());
                        evaluation.written(index, &values[index]).unwrap();
                    }
                    EvaluationAction::Compute => evaluation.compute().unwrap(),
                    EvaluationAction::Done => {
                        actual = Some(evaluation.finish().unwrap());
                        break;
                    }
                    action => panic!("unexpected evaluator action {source}: {action:?}"),
                }
            }
            let actual = actual.expect("evaluation finished");
            assert_eq!(actual.value.to_bits(), expected.to_bits(), "{source}");
            assert_eq!(
                values.len(),
                arithmetic.len(),
                "each real event/record has exactly one evaluated record"
            );
            assert!(visits > 0);
        }
    }
}
