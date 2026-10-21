//! Per-component syntax/type continuation, independent of reached color facts.
//!
//! Both arms of light-dark and every unselected fallback are checked by the
//! owning grammar pass. Resolution can later visit only the influencing arm.

use super::channels::{Component, Domain};
use super::expression::{
    ExpressionDescriptor, ExpressionTable, NumericAction, NumericGrammar, OperandReference,
    Variable,
};
use super::grammar::{ComponentRole, FunctionRecipe, ProfileSpace, RawComponent};
use super::keywords::{ColorFunction as F, Word};
use super::literal::{LiteralAction, LiteralConversion};
use super::protocol::{CssColorBank, CssColorProtocolError, CssColorSlot, LexicalTable};
use super::tokens::{ClassifiedToken, TokenAction, TokenReader};
use crate::numeric::bounded_expression::FailureKind;
use crate::tokenization::color_tokens::{NameKind, TokenKind};
use crate::{
    CssColorSyntax, CssNumericDimension as Dimension, CssNumericType,
    CssRelativeColorEnvironment as Environment,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Expectation {
    NumberPercentage,
    Hue,
    Percentage,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Variables {
    None,
    Relative(Environment),
    Profile,
}

/// Derived from the actual completed function record and its component ordinal.
/// It is neither a caller-provided validation flag nor a resolved color value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ComponentContext {
    expectation: Expectation,
    variables: Variables,
    allow_missing: bool,
}
impl ComponentContext {
    pub(crate) fn accepts_binding(self, binding: super::relative::Binding) -> bool {
        matches!(self.variables, Variables::Relative(environment) if environment == binding.environment())
    }
    pub(crate) fn for_recipe(
        recipe: FunctionRecipe,
        role: ComponentRole,
        ordinal: usize,
    ) -> Result<Self, CssColorProtocolError> {
        if recipe.syntax == CssColorSyntax::Legacy && recipe.origin.is_some() {
            return Err(CssColorProtocolError::WrongSlot);
        }
        let relative = recipe.origin.is_some();
        let environment = match recipe.function {
            F::Rgb => Some(Environment::Rgb),
            F::Hsl => Some(Environment::Hsl),
            F::Hwb => Some(Environment::Hwb),
            F::Lab => Some(Environment::Lab),
            F::Lch => Some(Environment::Lch),
            F::Oklab => Some(Environment::Oklab),
            F::Oklch => Some(Environment::Oklch),
            F::Alpha => Some(Environment::Alpha),
            F::Color => match recipe.space {
                Some(ProfileSpace::Predefined(space)) => Some(
                    if matches!(
                        space,
                        crate::PredefinedColorSpace::XyzD50 | crate::PredefinedColorSpace::XyzD65
                    ) {
                        Environment::Xyz(space)
                    } else {
                        Environment::PredefinedRgb(space)
                    },
                ),
                Some(ProfileSpace::Custom { .. }) => None,
                None => return Err(CssColorProtocolError::WrongSlot),
            },
            F::DeviceCmyk | F::Mix => None,
            F::LightDark | F::Contrast => return Err(CssColorProtocolError::WrongSlot),
        };
        let variables = if !relative {
            Variables::None
        } else if let Some(environment) = environment {
            if !environment.is_consistent() {
                return Err(CssColorProtocolError::WrongSlot);
            }
            Variables::Relative(environment)
        } else if matches!(recipe.space, Some(ProfileSpace::Custom { .. })) {
            Variables::Profile
        } else {
            return Err(CssColorProtocolError::WrongSlot);
        };
        let expectation = match role {
            ComponentRole::Weight if recipe.function == F::Mix => Expectation::Percentage,
            ComponentRole::Alpha => Expectation::NumberPercentage,
            ComponentRole::Hue => Expectation::Hue,
            ComponentRole::RelativeChannel if relative => {
                if ordinal == 0 && matches!(recipe.function, F::Hsl | F::Hwb)
                    || ordinal == 2 && matches!(recipe.function, F::Lch | F::Oklch)
                {
                    Expectation::Hue
                } else {
                    Expectation::NumberPercentage
                }
            }
            ComponentRole::ProfileChannel
                if matches!(recipe.space, Some(ProfileSpace::Custom { .. })) =>
            {
                Expectation::NumberPercentage
            }
            ComponentRole::Channel if !relative => {
                if recipe.syntax == CssColorSyntax::Legacy && recipe.function == F::Hsl {
                    Expectation::Percentage
                } else {
                    Expectation::NumberPercentage
                }
            }
            _ => return Err(CssColorProtocolError::WrongSlot),
        };
        Ok(Self {
            expectation,
            variables,
            allow_missing: recipe.syntax == CssColorSyntax::Modern && role != ComponentRole::Weight,
        })
    }
    /// The checked component of one converted literal. A `<number>` hue is
    /// degrees (Color 4 §4.3), so an authored number beyond binary64 takes the
    /// Values 4 §10.12 angle case, the nearest supported multiple of 360deg,
    /// exactly like an overflowing `<angle>`.
    pub(crate) fn literal(self, value: super::literal::LiteralValue) -> Option<Component> {
        let domain = self.domain(value.ty)?;
        let number =
            if self.expectation == Expectation::Hue && domain == Domain::Number && value.overflowed
            {
                crate::numeric::projection::closest_supported_value(
                    f64::INFINITY.copysign(value.value),
                    true,
                    true,
                )
            } else {
                value.value
            };
        Some(Component {
            domain,
            value: number,
        })
    }
    pub(crate) fn domain(self, ty: CssNumericType) -> Option<Domain> {
        match self.expectation {
            Expectation::Hue if ty.percent_hint().is_none() => {
                if ty.is_number() {
                    Some(Domain::Number)
                } else if ty.is(Dimension::Angle) {
                    Some(Domain::Degrees)
                } else {
                    None
                }
            }
            Expectation::Percentage
                if ty.is(Dimension::Percentage) && ty.percent_hint().is_none() =>
            {
                Some(Domain::Percentage)
            }
            Expectation::NumberPercentage => {
                if ty.is_number() {
                    Some(Domain::Number)
                } else if ty.is(Dimension::Percentage) {
                    Some(Domain::Percentage)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum CheckedComponent<'owner> {
    Literal(Component),
    Variable {
        token: OperandReference,
        binding: Variable,
    },
    Expression {
        table: ExpressionTable<'owner>,
        domain: Domain,
    },
}
/// The fixed payload stored alongside the actual raw component, never a
/// resolved palette/profile claim or a second component population.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CheckedScalar {
    Literal(Component),
    Variable {
        token: OperandReference,
        binding: Variable,
    },
    Expression {
        descriptor: ExpressionDescriptor,
        domain: Domain,
    },
}
impl PartialEq for CheckedScalar {
    fn eq(&self, other: &Self) -> bool {
        match (*self, *other) {
            (Self::Literal(a), Self::Literal(b)) => {
                a.domain == b.domain && a.value.to_bits() == b.value.to_bits()
            }
            (
                Self::Variable {
                    token: a,
                    binding: ab,
                },
                Self::Variable {
                    token: b,
                    binding: bb,
                },
            ) => a == b && ab == bb,
            (
                Self::Expression {
                    descriptor: a,
                    domain: ad,
                },
                Self::Expression {
                    descriptor: b,
                    domain: bd,
                },
            ) => a == b && ad == bd,
            _ => false,
        }
    }
}
impl CheckedComponent<'_> {
    pub(crate) fn scalar(self) -> CheckedScalar {
        match self {
            Self::Literal(value) => CheckedScalar::Literal(value),
            Self::Variable { token, binding } => CheckedScalar::Variable { token, binding },
            Self::Expression { table, domain } => CheckedScalar::Expression {
                descriptor: table.descriptor(),
                domain,
            },
        }
    }
}
impl CheckedScalar {
    pub(crate) const fn domain(self) -> Domain {
        match self {
            Self::Literal(value) => value.domain,
            Self::Variable { .. } => Domain::Number,
            Self::Expression { domain, .. } => domain,
        }
    }
    pub(crate) fn component<'owner>(
        self,
        input: &super::grammar::GrammarInput<'owner>,
    ) -> Result<CheckedComponent<'owner>, CssColorProtocolError> {
        Ok(match self {
            Self::Literal(value) => CheckedComponent::Literal(value),
            Self::Variable { token, binding } => CheckedComponent::Variable { token, binding },
            Self::Expression { descriptor, domain } => CheckedComponent::Expression {
                table: ExpressionTable::from_descriptor(input, descriptor)?,
                domain,
            },
        })
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ComponentFailureReason {
    Component,
    Hue,
    RelativeChannel,
}
impl ComponentFailureReason {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Component => "component",
            Self::Hue => "hue",
            Self::RelativeChannel => "relative channel",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SyntaxFailure {
    Component {
        site: usize,
        reason: ComponentFailureReason,
    },
    Numeric {
        kind: FailureKind,
        site: usize,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ComponentAction {
    Read { bank: CssColorBank, index: usize },
    Append { bank: CssColorBank, index: usize },
    Replace { bank: CssColorBank, index: usize },
    Compute,
    Done,
    Failed(SyntaxFailure),
    Stopped,
}
#[expect(
    clippy::large_enum_variant,
    reason = "Token, literal and math cursors are mutually exclusive; inline storage keeps the advertised receiver layout allocation-free and caller-accounted"
)]
enum State<'owner> {
    Token(TokenReader<'owner>),
    Header(ClassifiedToken),
    Literal(LiteralConversion<'owner>),
    Math(NumericGrammar<'owner>),
    Done(CheckedComponent<'owner>),
    Failed(SyntaxFailure),
    Stopped,
}
pub(crate) struct ComponentSyntax<'owner> {
    lexical: LexicalTable<'owner>,
    raw: RawComponent,
    context: ComponentContext,
    state: State<'owner>,
    scratch_end: usize,
    expression_end: usize,
}
impl<'owner> ComponentSyntax<'owner> {
    pub(crate) fn start(
        lexical: &LexicalTable<'owner>,
        raw: RawComponent,
        context: ComponentContext,
        scratch_end: usize,
        expression_end: usize,
    ) -> Result<Self, CssColorProtocolError> {
        if scratch_end < lexical.length() {
            return Err(CssColorProtocolError::WrongIndex);
        }
        let token = raw.source_token()?;
        let reader = TokenReader::start_at(lexical, token.index)?;
        Ok(Self {
            lexical: *lexical,
            raw,
            context,
            state: State::Token(reader),
            scratch_end,
            expression_end,
        })
    }
    pub(crate) fn action(&self) -> ComponentAction {
        match &self.state {
            State::Token(reader) => match reader.action() {
                TokenAction::ReadRecord { index } | TokenAction::ReadName { index } => {
                    ComponentAction::Read {
                        bank: CssColorBank::Algorithm,
                        index,
                    }
                }
                _ => ComponentAction::Compute,
            },
            State::Header(token) => ComponentAction::Read {
                bank: CssColorBank::Algorithm,
                index: token.index,
            },
            State::Literal(cursor) => match cursor.action() {
                LiteralAction::ReadSource { index }
                | LiteralAction::ReadScaled { index }
                | LiteralAction::ReadScratch { index }
                | LiteralAction::ReadHeader { index } => ComponentAction::Read {
                    bank: CssColorBank::Algorithm,
                    index,
                },
                LiteralAction::Append { index } => ComponentAction::Append {
                    bank: CssColorBank::Algorithm,
                    index,
                },
                LiteralAction::Replace { index } => ComponentAction::Replace {
                    bank: CssColorBank::Algorithm,
                    index,
                },
                LiteralAction::Compute | LiteralAction::Done(_) => ComponentAction::Compute,
                LiteralAction::Stopped => ComponentAction::Stopped,
            },
            State::Math(cursor) => match cursor.action() {
                NumericAction::ReadLexical { index }
                | NumericAction::ReadNumberHeader { index }
                | NumericAction::ReadNumber { index } => ComponentAction::Read {
                    bank: CssColorBank::Algorithm,
                    index,
                },
                NumericAction::AppendScratch { index } | NumericAction::AppendLeaf { index } => {
                    ComponentAction::Append {
                        bank: CssColorBank::Algorithm,
                        index,
                    }
                }
                NumericAction::ReplaceScratch { index } => ComponentAction::Replace {
                    bank: CssColorBank::Algorithm,
                    index,
                },
                NumericAction::ReadExpression { index } => ComponentAction::Read {
                    bank: CssColorBank::Arithmetic,
                    index,
                },
                NumericAction::AppendExpression { index } => ComponentAction::Append {
                    bank: CssColorBank::Arithmetic,
                    index,
                },
                NumericAction::Stopped => ComponentAction::Stopped,
                _ => ComponentAction::Compute,
            },
            State::Done(_) => ComponentAction::Done,
            State::Failed(failure) => ComponentAction::Failed(*failure),
            State::Stopped => ComponentAction::Stopped,
        }
    }
    pub(crate) fn read(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        if self.action() != (ComponentAction::Read { bank, index }) {
            return Err(CssColorProtocolError::WrongAction);
        }
        slot.qualify(self.lexical.owner, self.lexical.phase, index, bank)?;
        match &mut self.state {
            State::Token(reader) => reader.read(index, slot),
            State::Header(token) => {
                self.state = match self.raw {
                    RawComponent::Atom(_) => State::Literal(LiteralConversion::start(
                        &self.lexical,
                        *token,
                        slot,
                        self.scratch_end,
                    )?),
                    RawComponent::Authored { .. } => return Err(CssColorProtocolError::WrongSlot),
                    RawComponent::Math { .. } => State::Math(NumericGrammar::start(
                        &self.lexical,
                        *token,
                        slot,
                        self.scratch_end,
                        self.expression_end,
                    )?),
                };
                Ok(())
            }
            State::Literal(cursor) => cursor.read(index, slot),
            State::Math(cursor) => match cursor.action() {
                NumericAction::ReadLexical { .. } => cursor.read_lexical(index, slot),
                NumericAction::ReadNumberHeader { .. } | NumericAction::ReadNumber { .. } => {
                    cursor.read_number(index, slot)
                }
                NumericAction::ReadExpression { .. } => cursor.read_expression(index, slot),
                _ => Err(CssColorProtocolError::WrongAction),
            },
            _ => Err(CssColorProtocolError::WrongAction),
        }
    }
    pub(crate) fn record(
        &self,
        bank: CssColorBank,
        index: usize,
    ) -> Result<CssColorSlot<'owner>, CssColorProtocolError> {
        if !matches!(self.action(), ComponentAction::Append { bank: b, index: i } | ComponentAction::Replace { bank: b, index: i } if bank == b && index == i)
        {
            return Err(CssColorProtocolError::WrongAction);
        }
        match &self.state {
            State::Literal(cursor) => cursor.record(index),
            State::Math(cursor) => cursor.record(index),
            _ => Err(CssColorProtocolError::WrongAction),
        }
    }
    pub(crate) fn written(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        let expected = self.record(bank, index)?;
        slot.qualify(self.lexical.owner, self.lexical.phase, index, bank)?;
        if expected.bank() != bank {
            return Err(CssColorProtocolError::WrongSlot);
        }
        match &mut self.state {
            State::Literal(cursor) => {
                cursor.written(index, slot)?;
                self.scratch_end = cursor.scratch_end();
            }
            State::Math(cursor) => {
                cursor.written(index, slot)?;
                self.scratch_end = cursor.scratch_end();
                if bank == CssColorBank::Arithmetic {
                    self.expression_end = index
                        .checked_add(1)
                        .ok_or(CssColorProtocolError::Overflow)?;
                }
            }
            _ => return Err(CssColorProtocolError::WrongAction),
        }
        Ok(())
    }
    fn reject(&mut self, site: usize) {
        let reason = match (self.context.variables, self.context.expectation) {
            (Variables::Relative(_) | Variables::Profile, _) => {
                ComponentFailureReason::RelativeChannel
            }
            (_, Expectation::Hue) => ComponentFailureReason::Hue,
            _ => ComponentFailureReason::Component,
        };
        self.state = State::Failed(SyntaxFailure::Component { site, reason });
    }
    pub(crate) fn kernel(&self) -> super::CssColorKernel {
        match &self.state {
            State::Literal(cursor) => cursor.kernel(),
            State::Math(cursor) => cursor.kernel(),
            State::Token(reader) if reader.action() == TokenAction::Classify => {
                super::CssColorKernel::Classify
            }
            _ => super::CssColorKernel::Transition,
        }
    }
    pub(crate) fn compute(&mut self) -> Result<(), CssColorProtocolError> {
        if self.action() != ComponentAction::Compute {
            return Err(CssColorProtocolError::WrongAction);
        }
        match &mut self.state {
            State::Token(reader) if reader.action() == TokenAction::Classify => reader.classify(),
            State::Token(reader) => {
                let token = reader.current()?;
                let expected = self.raw.source_token()?;
                if token != expected {
                    return Err(CssColorProtocolError::WrongSlot);
                }
                match self.raw {
                    RawComponent::Math { .. } => self.state = State::Header(token),
                    RawComponent::Atom(_)
                        if token.word == Word::None && self.context.allow_missing =>
                    {
                        self.state = State::Done(CheckedComponent::Literal(Component::MISSING))
                    }
                    RawComponent::Atom(_)
                        if matches!(token.token.kind, TokenKind::Number { .. }) =>
                    {
                        self.state = State::Header(token)
                    }
                    RawComponent::Atom(_)
                        if matches!(
                            token.token.kind,
                            TokenKind::Name {
                                kind: NameKind::Ident,
                                ..
                            }
                        ) =>
                    {
                        let binding = match self.context.variables {
                            Variables::Relative(environment) => {
                                reader.relative_channel(environment)?.map(Variable::Channel)
                            }
                            Variables::Profile => {
                                let TokenKind::Name { units: name, .. } = token.token.kind else {
                                    unreachable!()
                                };
                                Some(Variable::Profile {
                                    name: super::protocol::RecordRange {
                                        start: name.start,
                                        end: name.end,
                                    },
                                })
                            }
                            Variables::None => None,
                        };
                        if let Some(binding) = binding {
                            self.state = State::Done(CheckedComponent::Variable {
                                token: OperandReference::Source(token),
                                binding,
                            });
                        } else {
                            self.reject(token.index);
                        }
                    }
                    _ => self.reject(token.index),
                }
                Ok(())
            }
            State::Literal(cursor) => {
                if let LiteralAction::Done(value) = cursor.action() {
                    if let Some(component) = self.context.literal(value) {
                        self.state = State::Done(CheckedComponent::Literal(component));
                    } else {
                        let RawComponent::Atom(token) = self.raw else {
                            unreachable!()
                        };
                        self.reject(token.index);
                    }
                    Ok(())
                } else {
                    cursor.compute()
                }
            }
            State::Math(cursor) => match cursor.action() {
                NumericAction::Done { ty, .. } => {
                    let RawComponent::Math {
                        opener,
                        end,
                        closed,
                    } = self.raw
                    else {
                        unreachable!()
                    };
                    if let Some(domain) = self.context.domain(ty) {
                        let table = cursor.finish()?;
                        if table.source_last_token()? != end || table.closed != closed {
                            return Err(CssColorProtocolError::WrongSlot);
                        }
                        self.state = State::Done(CheckedComponent::Expression { table, domain });
                    } else {
                        self.reject(opener.index);
                    }
                    Ok(())
                }
                NumericAction::Failed { kind, site } => {
                    self.state = State::Failed(SyntaxFailure::Numeric { kind, site });
                    Ok(())
                }
                NumericAction::ResolveIdentifier { token } => match self.context.variables {
                    Variables::Relative(environment)
                        if cursor.relative_channel(environment)?.is_some() =>
                    {
                        cursor.relative_identifier(environment)
                    }
                    Variables::Profile => cursor.profile_identifier(),
                    _ => {
                        self.reject(token.index);
                        Ok(())
                    }
                },
                _ => cursor.compute(),
            },
            _ => Err(CssColorProtocolError::WrongAction),
        }
    }
    pub(crate) fn finish(&self) -> Result<CheckedComponent<'owner>, CssColorProtocolError> {
        match self.state {
            State::Done(component) => Ok(component),
            _ => Err(CssColorProtocolError::Terminal),
        }
    }
    pub(crate) const fn scratch_end(&self) -> usize {
        self.scratch_end
    }
    pub(crate) const fn expression_end(&self) -> usize {
        self.expression_end
    }
    pub(crate) fn abandon(&mut self) {
        match &mut self.state {
            State::Token(reader) => reader.abandon(),
            State::Literal(cursor) => cursor.abandon(),
            State::Math(cursor) => cursor.abandon(),
            _ => {}
        }
        self.state = State::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use super::super::grammar::{ColorGrammar, GrammarAction, GrammarRecord, RecipeKind};
    use super::super::protocol::{
        ActionDescription, CssColorOperationOwner, Data, LexicalOperation,
    };
    use super::*;
    use crate::tokenization::bounded::InputKind;

    #[derive(Debug, PartialEq)]
    enum Shape {
        Literal(Domain, f64),
        Variable(Variable),
        Math(Domain, bool),
    }

    fn check(input: &str) -> Result<Vec<Shape>, SyntaxFailure> {
        let mut owner = CssColorOperationOwner::new();
        let mut lexical =
            LexicalOperation::start(&mut owner, InputKind::Utf8, input.len()).unwrap();
        let mut slots = Vec::new();
        loop {
            let mut action = lexical.next_action();
            match action.description() {
                ActionDescription::ReadInput { offset } => action
                    .read_input(offset, u16::from(input.as_bytes()[offset]))
                    .unwrap(),
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
                ActionDescription::Done => break,
                ActionDescription::Stopped => panic!("lexical stop"),
            }
        }
        let lexical = lexical.finish().unwrap();
        let mut grammar = ColorGrammar::start(&lexical, true);
        loop {
            match grammar.action() {
                GrammarAction::ReadLexical { index } => {
                    grammar.read_lexical(index, &slots[index]).unwrap()
                }
                GrammarAction::ReadFrame { index } => {
                    grammar.read_frame(index, &slots[index]).unwrap()
                }
                GrammarAction::ReadHex { index } => grammar.read_hex(index, &slots[index]).unwrap(),
                GrammarAction::Append { index } => {
                    slots.push(grammar.record(index).unwrap());
                    grammar.appended(index, &slots[index]).unwrap();
                }
                GrammarAction::Compute => grammar.compute().unwrap(),
                GrammarAction::Done { .. } => break,
                GrammarAction::Failed { source } => panic!("fixture color structure {:?}", source),
                GrammarAction::Stopped => panic!("grammar stop"),
            }
        }
        let mut components = Vec::new();
        // Test-only traversal of actual emitted records establishes each
        // contextual role. Expected domains below are literal source oracles.
        for slot in &slots {
            let Data::Grammar(GrammarRecord::Recipe(recipe)) = slot.data else {
                continue;
            };
            let RecipeKind::Function(function) = recipe.kind else {
                continue;
            };
            let mut chain = Vec::new();
            let mut index = function.components;
            while let Some(i) = index {
                let Data::Grammar(GrammarRecord::Component(value)) = slots[i].data else {
                    panic!("component link")
                };
                chain.push((value.component, value.role, value.ordinal));
                index = value.previous;
            }
            chain.reverse();
            components.extend(chain.into_iter().map(|(raw, role, ordinal)| {
                (
                    raw,
                    ComponentContext::for_recipe(function, role, ordinal).unwrap(),
                )
            }));
            if let Some(i) = function.alpha {
                let Data::Grammar(GrammarRecord::Component(value)) = slots[i].data else {
                    panic!("alpha link")
                };
                components.push((value.component, value.context));
            }
            let mut index = function.mix;
            while let Some(i) = index {
                let Data::Grammar(GrammarRecord::Mix(value)) = slots[i].data else {
                    panic!("mix link")
                };
                if let Some(i) = value.weight {
                    let Data::Grammar(GrammarRecord::Component(value)) = slots[i].data else {
                        panic!("weight link")
                    };
                    components.push((value.component, value.context));
                }
                index = value.previous;
            }
        }
        let mut arithmetic = Vec::new();
        let mut output = Vec::new();
        for (raw, context) in components {
            let mut cursor =
                ComponentSyntax::start(&lexical, raw, context, slots.len(), arithmetic.len())
                    .unwrap();
            loop {
                match cursor.action() {
                    ComponentAction::Read { bank, index } => {
                        let slot = match bank {
                            CssColorBank::Algorithm => &slots[index],
                            CssColorBank::Arithmetic => &arithmetic[index],
                            CssColorBank::Evaluation => panic!("no reached evaluation in syntax"),
                        };
                        cursor.read(bank, index, slot).unwrap();
                    }
                    ComponentAction::Append { bank, index } => {
                        let destination = match bank {
                            CssColorBank::Algorithm => &mut slots,
                            CssColorBank::Arithmetic => &mut arithmetic,
                            CssColorBank::Evaluation => panic!("syntax output bank"),
                        };
                        assert_eq!(index, destination.len());
                        destination.push(cursor.record(bank, index).unwrap());
                        cursor.written(bank, index, &destination[index]).unwrap();
                    }
                    ComponentAction::Replace { bank, index } => {
                        assert_eq!(bank, CssColorBank::Algorithm);
                        slots[index] = cursor.record(bank, index).unwrap();
                        cursor.written(bank, index, &slots[index]).unwrap();
                    }
                    ComponentAction::Compute => cursor.compute().unwrap(),
                    ComponentAction::Done => break,
                    ComponentAction::Failed(failure) => return Err(failure),
                    ComponentAction::Stopped => panic!("component stop"),
                }
            }
            assert_eq!(cursor.scratch_end(), slots.len());
            assert_eq!(cursor.expression_end(), arithmetic.len());
            output.push(match cursor.finish().unwrap() {
                CheckedComponent::Literal(value) => Shape::Literal(value.domain, value.value),
                CheckedComponent::Variable { token, binding } => {
                    assert!(token.index() < lexical.length());
                    Shape::Variable(binding)
                }
                CheckedComponent::Expression { table, domain } => {
                    assert_eq!(context.domain(table.ty), Some(domain));
                    Shape::Math(domain, table.closed)
                }
            });
            cursor.abandon();
            assert_eq!(cursor.action(), ComponentAction::Stopped);
            assert_eq!(cursor.finish().err(), Some(CssColorProtocolError::Terminal));
        }
        Ok(output)
    }

    #[test]
    fn function_roles_keep_number_percentage_angle_and_missing_distinct() {
        use Domain::*;
        assert_eq!(
            check("rgb(1 2% none / calc(1 / 2))").unwrap(),
            [
                Shape::Literal(Number, 1.0),
                Shape::Literal(Percentage, 2.0),
                Shape::Literal(Missing, 0.0),
                Shape::Math(Number, true)
            ]
        );
        assert_eq!(
            check("hsl(1turn, 20%, 30%, .5)").unwrap(),
            [
                Shape::Literal(Degrees, 360.0),
                Shape::Literal(Percentage, 20.0),
                Shape::Literal(Percentage, 30.0),
                Shape::Literal(Number, 0.5)
            ]
        );
        assert!(check("hsl(0, 20, 30%)").is_err());
        for input in [
            "rgb(1px 0 0)",
            "rgb(calc(1em) 0 0)",
            "lch(1 2 3%)",
            "rgb(1, 2, none)",
            "color-mix(red calc(1),blue)",
            "rgb(calc(1 + 2%) 0 0)",
        ] {
            assert!(check(input).is_err(), "{input}");
        }
        assert_eq!(
            check("color-mix(red calc(20%),blue)").unwrap(),
            [Shape::Math(Percentage, true)]
        );
        assert_eq!(
            check("rgb(calc(1em / 1em) 0 0)").unwrap(),
            [
                Shape::Math(Number, true),
                Shape::Literal(Number, 0.0),
                Shape::Literal(Number, 0.0)
            ]
        );
    }

    #[test]
    fn every_structural_branch_is_syntax_checked_before_reached_facts() {
        assert!(check("light-dark(red,rgb(1px 0 0))").is_err());
        assert!(check("light-dark(rgb(calc(1 + 2) 0 0), rgb(calc(1px) 0 0))").is_err());
        assert!(check("device-cmyk(0 1 1 0 / calc(20%))").is_ok());
        assert!(check("rgb(1 2 calc(3 /* end */").is_ok());
        assert_eq!(
            check("rgb(1 2 calc(3").unwrap().last(),
            Some(&Shape::Math(Domain::Number, false))
        );
    }

    #[test]
    fn relative_names_stay_symbolic_and_use_the_actual_environment() {
        let values = check("rgb(from red r calc(g * 2) b / alpha)").unwrap();
        assert_eq!(
            values,
            [
                Shape::Variable(Variable::Channel(crate::CssRelativeColorChannel::R)),
                Shape::Math(Domain::Number, true),
                Shape::Variable(Variable::Channel(crate::CssRelativeColorChannel::B)),
                Shape::Variable(Variable::Channel(crate::CssRelativeColorChannel::Alpha))
            ]
        );
        assert!(check("rgb(from red h g b)").is_err());
        assert!(check("hsl(from red calc(s + 1%) s l)").is_err());
        assert!(check("color(from red --profile é calc(up * 2) 0)").is_ok());
        assert!(check("rgb(calc(r) 0 0)").is_err());
    }
}
