//! Reached scalar evaluation over the checked component's real bank descriptor.
//!
//! Syntax checking has already covered all branches. Only an influencing
//! component creates this continuation, and no source is parsed a second time.

use super::channels::{Component, Domain, Error as ChannelError};
use super::component::CheckedComponent;
use super::expression::{EvaluationAction, Number, NumericEvaluation, Variable};
use super::relative::Binding;
use super::syntax_pass::ValidatedGrammar;
use crate::color::controlled::protocol::ColorProtocolError;
use crate::color::controlled::{CssColorBank, CssColorSlot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Protocol(ColorProtocolError),
    MissingOrigin,
    Channel(ChannelError),
    /// Metadata/name qualification is still owned by the reached profile
    /// continuation. No arbitrary-channel reverse conversion is fabricated.
    ReverseCustomOrigin {
        token: usize,
    },
    /// The existing fact envelope has no computed contextual numeric basis.
    UnitBasis {
        token: usize,
        unit: crate::numeric::CssNumericUnit,
    },
}
impl From<ColorProtocolError> for Error {
    fn from(error: ColorProtocolError) -> Self {
        Self::Protocol(error)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Action {
    ReadComponent { index: usize },
    Read { bank: CssColorBank, index: usize },
    Append { index: usize },
    Compute,
    Done,
    Stopped,
}
#[expect(
    clippy::large_enum_variant,
    reason = "Numeric evaluation is an exclusive continuation; inline evaluator registers preserve caller-accounted storage without an additional allocator protocol"
)]
enum State<'owner> {
    Header,
    Evaluation {
        cursor: NumericEvaluation<'owner>,
        domain: Domain,
    },
    Variable {
        token: usize,
        binding: Variable,
    },
    Done(Component),
    Stopped,
}
pub(crate) struct ComponentValue<'owner> {
    grammar: ValidatedGrammar<'owner>,
    index: usize,
    evaluation_start: usize,
    binding: Option<Binding>,
    state: State<'owner>,
}
impl<'owner> ComponentValue<'owner> {
    pub(crate) fn start(
        grammar: &ValidatedGrammar<'owner>,
        index: usize,
        evaluation_start: usize,
        binding: Option<Binding>,
    ) -> Result<Self, ColorProtocolError> {
        if index < grammar.table.start || index >= grammar.table.end {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(Self {
            grammar: ValidatedGrammar {
                table: grammar.table,
                expression_end: grammar.expression_end,
            },
            index,
            evaluation_start,
            binding,
            state: State::Header,
        })
    }
    pub(crate) fn action(&self) -> Action {
        match &self.state {
            State::Header => Action::ReadComponent { index: self.index },
            State::Evaluation { cursor, .. } => match cursor.action() {
                EvaluationAction::ReadExpression { index } => Action::Read {
                    bank: CssColorBank::Arithmetic,
                    index,
                },
                EvaluationAction::ReadLeaf { index } => Action::Read {
                    bank: CssColorBank::Algorithm,
                    index,
                },
                EvaluationAction::ReadPrevious { index }
                | EvaluationAction::ReadOperand { index } => Action::Read {
                    bank: CssColorBank::Evaluation,
                    index,
                },
                EvaluationAction::Append { index } => Action::Append { index },
                EvaluationAction::Stopped => Action::Stopped,
                _ => Action::Compute,
            },
            State::Variable { .. } => Action::Compute,
            State::Done(_) => Action::Done,
            State::Stopped => Action::Stopped,
        }
    }
    pub(crate) fn read_component(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        if self.action() != (Action::ReadComponent { index }) {
            return Err(ColorProtocolError::WrongAction.into());
        }
        let record = self.grammar.component(index, slot)?;
        let component = record
            .checked
            .ok_or(ColorProtocolError::WrongSlot)?
            .component(&self.grammar.table.input)?;
        if self
            .binding
            .is_some_and(|binding| !record.context.accepts_binding(binding))
        {
            return Err(ColorProtocolError::WrongSlot.into());
        }
        self.state = match component {
            CheckedComponent::Literal(value) => State::Done(value),
            CheckedComponent::Variable { token, binding } => State::Variable {
                token: token.index(),
                binding,
            },
            CheckedComponent::Expression { table, domain } => State::Evaluation {
                cursor: NumericEvaluation::start(&table, self.evaluation_start)?,
                domain,
            },
        };
        Ok(())
    }
    pub(crate) fn read(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        if self.action() != (Action::Read { bank, index }) {
            return Err(ColorProtocolError::WrongAction.into());
        }
        slot.qualify(
            self.grammar.table.input.records().owner,
            self.grammar.table.input.records().phase,
            index,
            bank,
        )?;
        let State::Evaluation { cursor, .. } = &mut self.state else {
            return Err(ColorProtocolError::WrongAction.into());
        };
        match cursor.action() {
            EvaluationAction::ReadExpression { .. } => cursor.read_expression(index, slot)?,
            EvaluationAction::ReadLeaf { .. } => cursor.read_leaf(index, slot)?,
            EvaluationAction::ReadPrevious { .. } | EvaluationAction::ReadOperand { .. } => {
                cursor.read_value(index, slot)?
            }
            _ => return Err(ColorProtocolError::WrongAction.into()),
        }
        Ok(())
    }
    pub(crate) fn authored_site(&self) -> Option<usize> {
        match &self.state {
            State::Evaluation { cursor, .. } => cursor.authored_site(),
            _ => None,
        }
    }
    pub(crate) fn record(&self, index: usize) -> Result<CssColorSlot<'owner>, Error> {
        let State::Evaluation { cursor, .. } = &self.state else {
            return Err(ColorProtocolError::WrongAction.into());
        };
        Ok(cursor.record(index)?)
    }
    pub(crate) fn written(&mut self, index: usize, slot: &CssColorSlot<'_>) -> Result<(), Error> {
        let State::Evaluation { cursor, .. } = &mut self.state else {
            return Err(ColorProtocolError::WrongAction.into());
        };
        cursor.written(index, slot)?;
        Ok(())
    }
    pub(crate) fn kernel(&self) -> crate::color::controlled::CssColorKernel {
        match &self.state {
            State::Variable { .. } => crate::color::controlled::CssColorKernel::RelativeChannel,
            State::Evaluation { cursor, .. } => match cursor.action() {
                EvaluationAction::ResolveVariable { .. } => {
                    crate::color::controlled::CssColorKernel::RelativeChannel
                }
                _ => cursor.kernel(),
            },
            _ => crate::color::controlled::CssColorKernel::Transition,
        }
    }
    pub(crate) fn compute(&mut self) -> Result<(), Error> {
        if self.action() != Action::Compute {
            return Err(ColorProtocolError::WrongAction.into());
        }
        match &mut self.state {
            State::Variable { token, binding } => {
                let component = match binding {
                    Variable::Channel(channel) => self
                        .binding
                        .ok_or(Error::MissingOrigin)?
                        .channel(*channel, false)
                        .map_err(Error::Channel)?,
                    Variable::Profile { .. } => {
                        return Err(Error::ReverseCustomOrigin { token: *token });
                    }
                };
                self.state = State::Done(component);
                Ok(())
            }
            State::Evaluation { cursor, domain } => match cursor.action() {
                EvaluationAction::ResolveVariable { token, binding } => {
                    let component = match binding {
                        Variable::Channel(channel) => self
                            .binding
                            .ok_or(Error::MissingOrigin)?
                            .channel(channel, true)
                            .map_err(Error::Channel)?,
                        Variable::Profile { .. } => {
                            return Err(Error::ReverseCustomOrigin { token });
                        }
                    };
                    cursor.variable(
                        token,
                        Number {
                            value: component.value,
                            ty: crate::numeric::CssNumericType::NUMBER,
                        },
                    )?;
                    Ok(())
                }
                EvaluationAction::ResolveUnit { token, unit } => {
                    cursor.unavailable_unit(token, unit)?;
                    Err(Error::UnitBasis { token, unit })
                }
                EvaluationAction::Done => {
                    let number = cursor.finish()?;
                    self.state = State::Done(Component {
                        domain: *domain,
                        value: number.value,
                    });
                    Ok(())
                }
                EvaluationAction::Compute => {
                    cursor.compute()?;
                    Ok(())
                }
                _ => Err(ColorProtocolError::WrongAction.into()),
            },
            _ => Err(ColorProtocolError::WrongAction.into()),
        }
    }
    pub(crate) fn finish(&self) -> Result<Component, ColorProtocolError> {
        if let State::Done(value) = self.state {
            Ok(value)
        } else {
            Err(ColorProtocolError::Terminal)
        }
    }
    pub(crate) fn abandon(&mut self) {
        if let State::Evaluation { cursor, .. } = &mut self.state {
            cursor.abandon();
        }
        self.state = State::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use super::super::grammar::{FunctionRecipe, GrammarRecord, RecipeKind};
    use super::super::syntax_pass::tests::{Fixture, fixture};
    use super::*;
    use crate::color::CssRelativeColorEnvironment as Environment;
    use crate::color::controlled::CssColorOperationOwner;

    fn function(fixture: &Fixture<'_>) -> FunctionRecipe {
        let GrammarRecord::Recipe(recipe) = fixture
            .grammar
            .table
            .record(
                fixture.grammar.table.root,
                &fixture.slots[fixture.grammar.table.root],
            )
            .unwrap()
        else {
            panic!("root recipe")
        };
        let RecipeKind::Function(function) = recipe.kind else {
            panic!("function root")
        };
        function
    }
    fn evaluate<'owner>(
        fixture: &Fixture<'owner>,
        index: usize,
        values: &mut Vec<CssColorSlot<'owner>>,
        binding: Option<Binding>,
    ) -> Result<Component, Error> {
        let mut cursor =
            ComponentValue::start(&fixture.grammar, index, values.len(), binding).unwrap();
        loop {
            let step = match cursor.action() {
                Action::ReadComponent { index } => {
                    cursor.read_component(index, &fixture.slots[index])
                }
                Action::Read { bank, index } => {
                    let slot = match bank {
                        CssColorBank::Algorithm => &fixture.slots[index],
                        CssColorBank::Arithmetic => &fixture.arithmetic[index],
                        CssColorBank::Evaluation => &values[index],
                    };
                    cursor.read(bank, index, slot)
                }
                Action::Append { index } => {
                    assert_eq!(index, values.len());
                    values.push(cursor.record(index).unwrap());
                    cursor.written(index, &values[index])
                }
                Action::Compute => cursor.compute(),
                Action::Done => {
                    let value = cursor.finish().unwrap();
                    cursor.abandon();
                    assert_eq!(cursor.action(), Action::Stopped);
                    return Ok(value);
                }
                Action::Stopped => panic!("unexpected terminal"),
            };
            if let Err(error) = step {
                cursor.abandon();
                assert_eq!(cursor.action(), Action::Stopped);
                return Err(error);
            }
        }
    }
    #[test]
    fn actual_component_spans_evaluate_without_unrelated_prefix_placeholders() {
        let mut owner = CssColorOperationOwner::new();
        let fixture = fixture(&mut owner, "rgb(calc(1 + 2) calc(20%) 0)").unwrap();
        let function = function(&fixture);
        let mut values = Vec::new();
        assert_eq!(
            evaluate(
                &fixture,
                function.component_index(1).unwrap(),
                &mut values,
                None
            ),
            Ok(Component {
                domain: Domain::Percentage,
                value: 20.0
            })
        );
        let span_length = |ordinal| {
            let index = function.component_index(ordinal).unwrap();
            let record = fixture
                .grammar
                .component(index, &fixture.slots[index])
                .unwrap();
            let CheckedComponent::Expression { table, .. } = record
                .checked
                .unwrap()
                .component(&fixture.grammar.table.input)
                .unwrap()
            else {
                panic!("expression component")
            };
            if ordinal == 1 {
                assert!(table.start > 0);
            }
            table.length - table.start
        };
        let second_length = span_length(1);
        assert_eq!(
            values.len(),
            second_length,
            "only the selected expression span is emitted"
        );
        assert_eq!(
            evaluate(
                &fixture,
                function.component_index(0).unwrap(),
                &mut values,
                None
            ),
            Ok(Component::number(3.0))
        );
        assert_eq!(values.len(), second_length + span_length(0));
        assert_eq!(
            evaluate(
                &fixture,
                function.component_index(2).unwrap(),
                &mut values,
                None
            ),
            Ok(Component::number(0.0))
        );
        assert_eq!(
            values.len(),
            second_length + span_length(0),
            "literal creates no evaluation record"
        );
    }
    #[test]
    fn relative_binding_uses_the_actual_origin_environment_and_alpha() {
        let mut owner = CssColorOperationOwner::new();
        let fixture = fixture(&mut owner, "rgb(from red r calc(g + 1) b / alpha)").unwrap();
        let function = function(&fixture);
        let origin = function.origin.unwrap();
        let GrammarRecord::Recipe(recipe) = fixture
            .grammar
            .table
            .record(origin, &fixture.slots[origin])
            .unwrap()
        else {
            panic!("origin recipe")
        };
        assert_eq!(
            recipe.kind,
            RecipeKind::Leaf(super::super::grammar::Leaf::Named([255, 0, 0]))
        );
        let binding = Binding::new(
            Environment::Rgb,
            surgeist_color::Color::from_srgba8([255, 0, 0, 255]),
        )
        .unwrap();
        let mut values = Vec::new();
        for (ordinal, expected) in [255.0, 1.0, 0.0].into_iter().enumerate() {
            assert_eq!(
                evaluate(
                    &fixture,
                    function.component_index(ordinal).unwrap(),
                    &mut values,
                    Some(binding)
                ),
                Ok(Component::number(expected))
            );
        }
        assert_eq!(
            evaluate(
                &fixture,
                function.alpha.unwrap(),
                &mut values,
                Some(binding)
            ),
            Ok(Component::number(1.0))
        );
        let wrong = Binding::new(Environment::Hsl, surgeist_color::Color::opaque_black()).unwrap();
        assert_eq!(
            evaluate(
                &fixture,
                function.component_index(0).unwrap(),
                &mut values,
                Some(wrong)
            ),
            Err(Error::Protocol(ColorProtocolError::WrongSlot))
        );
        assert_eq!(
            evaluate(
                &fixture,
                function.component_index(0).unwrap(),
                &mut values,
                None
            ),
            Err(Error::MissingOrigin)
        );
    }
    #[test]
    fn contextual_basis_is_reached_after_valid_numeric_type_not_guessed() {
        let mut owner = CssColorOperationOwner::new();
        let fixture = fixture(&mut owner, "rgb(calc(1em / 1em) 0 0)").unwrap();
        let function = function(&fixture);
        let mut values = Vec::new();
        assert!(matches!(
            evaluate(
                &fixture,
                function.component_index(0).unwrap(),
                &mut values,
                None
            ),
            Err(Error::UnitBasis {
                unit: crate::numeric::CssNumericUnit::Length(crate::numeric::CssLengthUnit::Em),
                ..
            })
        ));
        assert!(values.is_empty());
        let mut other = CssColorOperationOwner::new();
        assert!(
            super::super::syntax_pass::tests::fixture(&mut other, "rgb(calc(1em) 0 0)").is_err()
        );
    }
    #[test]
    fn foreign_records_do_not_advance_the_pending_real_component_read() {
        let mut owner = CssColorOperationOwner::new();
        let mut other = CssColorOperationOwner::new();
        let fixture = fixture(&mut owner, "rgb(1 2 3)").unwrap();
        let foreign = super::super::syntax_pass::tests::fixture(&mut other, "rgb(1 2 3)").unwrap();
        let index = function(&fixture).component_index(0).unwrap();
        let mut cursor = ComponentValue::start(&fixture.grammar, index, 0, None).unwrap();
        assert_eq!(
            cursor.read_component(index, &foreign.slots[index]),
            Err(Error::Protocol(ColorProtocolError::ForeignSlot))
        );
        assert_eq!(cursor.action(), Action::ReadComponent { index });
        cursor.read_component(index, &fixture.slots[index]).unwrap();
        assert_eq!(cursor.finish(), Ok(Component::number(1.0)));
        cursor.abandon();
    }
}
