//! One original typed scalar into the existing checked component payload.

use super::super::{
    channels::Component,
    component::{CheckedScalar, ComponentContext, SyntaxFailure},
    expression::{OperandReference, Variable},
    literal::{LiteralAction, LiteralConversion},
    protocol::{
        ColorProtocolError, CssColorBank, CssColorOperationOwner, CssColorSlot, Data, RecordTable,
    },
};
use super::position::Position;
use super::{
    CssColorPathStep,
    input::Operand,
    numeric::{self, NumericLowering},
    site::SiteRecord,
    spelling::{self, NameSpelling, NumberSpelling},
};
use crate::syntax::color::operands::Scalar;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Action {
    ReadHeader,
    ReadPayload { bytes: usize },
    ReadLiteralHeader,
    ReadNameHeader,
    ReadByte { offset: usize },
    Numeric(numeric::Action),
    Read { bank: CssColorBank, index: usize },
    Append { bank: CssColorBank, index: usize },
    Replace { bank: CssColorBank, index: usize },
    Compute,
    Done,
    Failed(SyntaxFailure),
}
#[expect(
    clippy::large_enum_variant,
    reason = "Exact literal conversion and typed numerical traversal are exclusive, inline, caller-accounted continuations"
)]
enum State<'css> {
    Header,
    Payload {
        bytes: usize,
    },
    Site {
        value: Scalar<'css>,
        record: SiteRecord<'css>,
    },
    Value(Scalar<'css>),
    Number(NumberSpelling<'css>),
    NameHeader(&'css crate::CssColorProfileComponentName),
    Name(NameSpelling<'css>),
    ReadNumber {
        record: usize,
    },
    Convert(LiteralConversion<'css>),
    Numeric(NumericLowering<'css>),
    Done(CheckedScalar),
    Failed(SyntaxFailure),
}
pub(super) struct ScalarInput<'css> {
    pub(super) operand: Operand<'css>,
    pub(super) parent_site: usize,
    pub(super) depth: usize,
    pub(super) edge: CssColorPathStep,
    pub(super) context: ComponentContext,
}
pub(super) struct ScalarLowering<'css> {
    owner: &'css CssColorOperationOwner,
    phase: usize,
    operand: Operand<'css>,
    parent_site: usize,
    depth: usize,
    edge: CssColorPathStep,
    site: Option<usize>,
    context: ComponentContext,
    algorithm_end: usize,
    arithmetic_end: usize,
    state: State<'css>,
    position: Position,
}
impl<'css> ScalarLowering<'css> {
    pub(super) fn start(
        owner: &'css CssColorOperationOwner,
        phase: usize,
        input: ScalarInput<'css>,
        algorithm_end: usize,
        arithmetic_end: usize,
    ) -> Self {
        let ScalarInput {
            operand,
            parent_site,
            depth,
            edge,
            context,
        } = input;
        Self {
            owner,
            phase,
            operand,
            parent_site,
            depth,
            edge,
            site: None,
            context,
            algorithm_end,
            arithmetic_end,
            state: State::Header,
            position: Position {
                site: algorithm_end,
                depth,
                edge: Some(edge),
            },
        }
    }
    pub(super) fn action(&self) -> Action {
        match &self.state {
            State::Header => Action::ReadHeader,
            State::Payload { bytes } => Action::ReadPayload { bytes: *bytes },
            State::Site { .. } => Action::Append {
                bank: CssColorBank::Algorithm,
                index: self.algorithm_end,
            },
            State::NameHeader(_) => Action::ReadNameHeader,
            State::Number(writer) => spelling_action(writer.action()),
            State::Name(writer) => spelling_action(writer.action()),
            State::ReadNumber { record } => Action::Read {
                bank: CssColorBank::Algorithm,
                index: *record,
            },
            State::Convert(cursor) => match cursor.action() {
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
            State::Numeric(cursor) => match cursor.action() {
                numeric::Action::Done => Action::Compute,
                numeric::Action::Read { bank, index } => Action::Read { bank, index },
                numeric::Action::Append { bank, index } => Action::Append { bank, index },
                numeric::Action::Replace { bank, index } => Action::Replace { bank, index },
                numeric::Action::Compute => Action::Compute,
                action => Action::Numeric(action),
            },
            State::Value(Scalar::Literal { .. }) => Action::ReadLiteralHeader,
            State::Value(_) => Action::Compute,
            State::Done(_) => Action::Done,
            State::Failed(f) => Action::Failed(*f),
        }
    }
    pub(super) fn input_bytes(&self) -> Result<usize, ColorProtocolError> {
        Ok(match &self.state {
            State::Header => self.operand.header_bytes(),
            State::Payload { bytes } => *bytes,
            State::Value(Scalar::Literal { .. }) => std::mem::size_of::<crate::CssComponentValue>(),
            State::NameHeader(_) => std::mem::size_of::<crate::CssColorProfileComponentName>(),
            State::Numeric(cursor) => cursor.input_bytes()?,
            State::Number(_) | State::Name(_)
                if matches!(self.action(), Action::ReadByte { .. }) =>
            {
                1
            }
            _ => 0,
        })
    }
    pub(super) fn read_authored(&mut self) -> Result<(), ColorProtocolError> {
        match &mut self.state {
            State::Header => {
                let bytes = self.operand.scalar_bytes()?;
                self.state = State::Payload { bytes };
                Ok(())
            }
            State::Payload { .. } => {
                let value = self.operand.scalar()?;
                let origin = match value {
                    Scalar::Missing { origin } => origin,
                    Scalar::Literal { origin, .. }
                    | Scalar::Variable { origin, .. }
                    | Scalar::ProfileReference { origin, .. } => Some(origin),
                    Scalar::Expression(_) => None,
                };
                self.algorithm_end
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                self.state = State::Site {
                    value,
                    record: SiteRecord {
                        operand: self.operand,
                        parent: Some(self.parent_site),
                        edge: Some(self.edge),
                        depth: self.depth,
                        origin,
                    },
                };
                Ok(())
            }
            State::NameHeader(name) => {
                self.state = State::Name(NameSpelling::start(
                    self.owner,
                    self.phase,
                    name.as_str(),
                    self.site.ok_or(ColorProtocolError::WrongSlot)?,
                    self.algorithm_end,
                ));
                Ok(())
            }
            State::Value(Scalar::Literal {
                numeric,
                unit,
                percentage,
                ..
            }) => {
                self.state = State::Number(NumberSpelling::start(
                    self.owner,
                    self.phase,
                    *numeric,
                    *unit,
                    *percentage,
                    self.site.ok_or(ColorProtocolError::WrongSlot)?,
                    self.algorithm_end,
                ));
                Ok(())
            }
            State::Number(writer) => writer.read_byte(),
            State::Name(writer) => writer.read_byte(),
            State::Numeric(cursor) => cursor.read_authored(),
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
            State::ReadNumber { record } => {
                let table = RecordTable {
                    owner: self.owner,
                    phase: self.phase,
                    length: self.algorithm_end,
                };
                self.state = State::Convert(LiteralConversion::start_authored(
                    &table,
                    *record,
                    slot,
                    self.algorithm_end,
                )?);
                Ok(())
            }
            State::Convert(cursor) => cursor.read(index, slot),
            State::Numeric(cursor) => cursor.read(bank, index, slot),
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
            State::Site { record, .. } => Ok(CssColorSlot::new(
                self.owner,
                self.phase,
                index,
                Data::AuthoredSite(*record),
            )),
            State::Number(writer) => writer.record(index),
            State::Name(writer) => writer.record(index),
            State::Convert(cursor) => cursor.record(index),
            State::Numeric(cursor) => cursor.record(bank, index),
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    pub(super) fn written(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if !matches!(self.action(),Action::Append {bank:b,index:i}|Action::Replace {bank:b,index:i} if b==bank && i==index)
        {
            return Err(ColorProtocolError::WrongAction);
        }
        slot.qualify(self.owner, self.phase, index, bank)?;
        match &mut self.state {
            State::Site { value, record } => {
                let Data::AuthoredSite(actual) = slot.data else {
                    return Err(ColorProtocolError::WrongSlot);
                };
                if !record.identical(actual) {
                    return Err(ColorProtocolError::WrongSlot);
                }
                self.algorithm_end = self
                    .algorithm_end
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                self.site = Some(index);
                self.state = State::Value(*value);
                Ok(())
            }
            State::Number(writer) => {
                writer.written(index, slot)?;
                self.algorithm_end = writer.algorithm_end();
                Ok(())
            }
            State::Name(writer) => {
                writer.written(index, slot)?;
                self.algorithm_end = writer.algorithm_end();
                Ok(())
            }
            State::Convert(cursor) => {
                cursor.written(index, slot)?;
                self.algorithm_end = cursor.scratch_end();
                Ok(())
            }
            State::Numeric(cursor) => {
                cursor.written(bank, index, slot)?;
                self.algorithm_end = cursor.algorithm_end();
                self.arithmetic_end = cursor.arithmetic_end();
                Ok(())
            }
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    pub(super) fn kernel(&self) -> crate::color::CssColorKernel {
        match &self.state {
            State::Convert(cursor) => cursor.kernel(),
            State::Numeric(cursor) => cursor.kernel(),
            _ => crate::color::CssColorKernel::Transition,
        }
    }
    pub(super) fn compute(&mut self) -> Result<(), ColorProtocolError> {
        if self.action() != Action::Compute {
            return Err(ColorProtocolError::WrongAction);
        }
        let site = self.site.ok_or(ColorProtocolError::WrongSlot)?;
        match &mut self.state {
            State::Value(value) => {
                match *value {
                    Scalar::Missing { .. } => {
                        self.state = State::Done(CheckedScalar::Literal(Component::MISSING))
                    }
                    Scalar::Literal { .. } => return Err(ColorProtocolError::WrongAction),
                    Scalar::Variable { channel, .. } => {
                        self.state = State::Done(CheckedScalar::Variable {
                            token: OperandReference::Authored { index: site, site },
                            binding: Variable::Channel(channel),
                        })
                    }
                    Scalar::ProfileReference { name, .. } => self.state = State::NameHeader(name),
                    Scalar::Expression(node) => {
                        self.state = State::Numeric(NumericLowering::start(
                            self.owner,
                            self.phase,
                            node,
                            site,
                            self.depth,
                            self.algorithm_end,
                            self.arithmetic_end,
                        ))
                    }
                };
                Ok(())
            }
            State::Number(writer) => {
                if let spelling::Action::Done { record } = writer.action() {
                    self.state = State::ReadNumber { record };
                    Ok(())
                } else {
                    writer.compute()
                }
            }
            State::Name(writer) => {
                if let spelling::Action::Done { record } = writer.action() {
                    let name = super::super::protocol::RecordRange {
                        start: writer.units_start(),
                        end: record,
                    };
                    self.state = State::Done(CheckedScalar::Variable {
                        token: OperandReference::Authored {
                            index: record,
                            site,
                        },
                        binding: Variable::Profile { name },
                    });
                    Ok(())
                } else {
                    writer.compute()
                }
            }
            State::Convert(cursor) => {
                if let LiteralAction::Done(value) = cursor.action() {
                    self.state = match self.context.literal(value) {
                        Some(component) => State::Done(CheckedScalar::Literal(component)),
                        None => State::Failed(SyntaxFailure::Component {
                            site,
                            reason: super::super::component::ComponentFailureReason::Component,
                        }),
                    };
                    Ok(())
                } else {
                    cursor.compute()
                }
            }
            State::Numeric(cursor) => {
                if cursor.action() == numeric::Action::Done {
                    let descriptor = cursor.finish()?;
                    self.position = cursor.position();
                    self.state = match self.context.domain(descriptor.numeric_type()) {
                        Some(domain) => {
                            State::Done(CheckedScalar::Expression { descriptor, domain })
                        }
                        None => State::Failed(SyntaxFailure::Numeric {
                            site,
                            kind:
                                crate::numeric::bounded_expression::FailureKind::InvalidArgumentType,
                        }),
                    };
                    Ok(())
                } else {
                    cursor.compute()
                }
            }
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    pub(super) fn finish(&self) -> Result<(usize, CheckedScalar), ColorProtocolError> {
        let State::Done(value) = self.state else {
            return Err(ColorProtocolError::Terminal);
        };
        Ok((self.site.ok_or(ColorProtocolError::WrongSlot)?, value))
    }
    pub(super) fn position(&self) -> Position {
        match &self.state {
            State::Numeric(cursor) => cursor.position(),
            _ => self.position,
        }
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
