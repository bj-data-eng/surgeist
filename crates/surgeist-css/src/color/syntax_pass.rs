//! One linear grammar-bank pass; all component syntax precedes reached facts.
//!
//! Numeric expression storage is appended to the same phase's arithmetic bank.
//! No recipe/source reparse, component lookup map or chain-prefix scan is used.

use super::component::{ComponentAction, ComponentSyntax, SyntaxFailure};
use super::grammar::{ComponentRecord, GrammarRecord, GrammarTable, RecipeKind};
use crate::color::controlled::protocol::{ColorProtocolError, Data};
use crate::color::controlled::{CssColorBank, CssColorSlot};

#[expect(
    clippy::large_enum_variant,
    reason = "Only the current component cursor is live; keeping it inline preserves caller-accounted receiver storage instead of unadmitted heap allocation"
)]
enum State<'owner> {
    Record,
    Component {
        record: ComponentRecord,
        cursor: ComponentSyntax<'owner>,
    },
    LegacyRgb {
        index: usize,
        remaining: usize,
        domains: [super::channels::Domain; 3],
        sites: [usize; 3],
    },
    Write(ComponentRecord),
    Done,
    Failed(SyntaxFailure),
    Stopped,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SyntaxAction {
    ReadRecord { index: usize },
    Component(ComponentAction),
    Replace { index: usize },
    Done,
    ReadLegacyRgb { index: usize },
    Failed(SyntaxFailure),
    Stopped,
}
pub(crate) struct SyntaxPass<'owner> {
    table: GrammarTable<'owner>,
    index: usize,
    scratch_end: usize,
    expression_end: usize,
    state: State<'owner>,
}
pub(crate) struct ValidatedGrammar<'owner> {
    pub(crate) table: GrammarTable<'owner>,
    pub(crate) expression_end: usize,
}
impl ValidatedGrammar<'_> {
    pub(crate) fn component(
        &self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<ComponentRecord, ColorProtocolError> {
        let GrammarRecord::Component(record) = self.table.record(index, slot)? else {
            return Err(ColorProtocolError::WrongSlot);
        };
        let checked = record.checked.ok_or(ColorProtocolError::WrongSlot)?;
        if let super::component::CheckedComponent::Expression { table, .. } =
            checked.component(&self.table.input)?
            && table.length > self.expression_end
        {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(record)
    }
}
impl<'owner> SyntaxPass<'owner> {
    pub(crate) fn start(
        table: GrammarTable<'owner>,
        scratch_end: usize,
        expression_end: usize,
    ) -> Result<Self, ColorProtocolError> {
        if scratch_end < table.end || table.root < table.start || table.root >= table.end {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(Self {
            table,
            index: table.start,
            scratch_end,
            expression_end,
            state: State::Record,
        })
    }
    pub(crate) fn action(&self) -> SyntaxAction {
        match &self.state {
            State::Record => SyntaxAction::ReadRecord { index: self.index },
            State::Component { cursor, .. } => SyntaxAction::Component(cursor.action()),
            State::Write(_) => SyntaxAction::Replace { index: self.index },
            State::Done => SyntaxAction::Done,
            State::LegacyRgb { index, .. } => SyntaxAction::ReadLegacyRgb { index: *index },
            State::Failed(failure) => SyntaxAction::Failed(*failure),
            State::Stopped => SyntaxAction::Stopped,
        }
    }
    fn advance(&mut self) -> Result<(), ColorProtocolError> {
        let next = self
            .index
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        self.index = next;
        self.state = if next == self.table.end {
            State::Done
        } else {
            State::Record
        };
        Ok(())
    }
    pub(crate) fn read_record(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if self.action() != (SyntaxAction::ReadRecord { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        let record = self.table.record(index, slot)?;
        self.index
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        if let GrammarRecord::Component(record) = record {
            if record.checked.is_some() {
                return Err(ColorProtocolError::WrongSlot);
            }
            let cursor = ComponentSyntax::start(
                &self.table.input.source()?,
                record.component,
                record.context,
                self.scratch_end,
                self.expression_end,
            )?;
            self.state = State::Component { record, cursor };
            Ok(())
        } else if let GrammarRecord::Recipe(recipe) = record
            && let RecipeKind::Function(function) = recipe.kind
            && function.function == super::keywords::ColorFunction::Rgb
            && function.syntax == crate::color::CssColorSyntax::Legacy
        {
            if function.count != 3 {
                return Err(ColorProtocolError::WrongSlot);
            }
            let index = function.components.ok_or(ColorProtocolError::WrongSlot)?;
            self.state = State::LegacyRgb {
                index,
                remaining: 3,
                domains: [super::channels::Domain::Missing; 3],
                sites: [0; 3],
            };
            Ok(())
        } else {
            self.advance()
        }
    }
    /// The legacy RGB relationship has exactly three checked records, distinct
    /// from a variadic profile chain or a per-prefix rescan.
    pub(crate) fn read_legacy_rgb(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        let State::LegacyRgb {
            index: expected,
            remaining,
            mut domains,
            mut sites,
        } = self.state
        else {
            return Err(ColorProtocolError::WrongAction);
        };
        if index != expected {
            return Err(ColorProtocolError::WrongIndex);
        }
        let GrammarRecord::Component(record) = self.table.record(index, slot)? else {
            return Err(ColorProtocolError::WrongSlot);
        };
        let ordinal = remaining
            .checked_sub(1)
            .ok_or(ColorProtocolError::WrongIndex)?;
        if record.ordinal != ordinal || record.role != super::grammar::ComponentRole::Channel {
            return Err(ColorProtocolError::WrongSlot);
        }
        domains[ordinal] = record
            .checked
            .ok_or(ColorProtocolError::WrongSlot)?
            .domain();
        sites[ordinal] = match record.component {
            super::grammar::RawComponent::Atom(token)
            | super::grammar::RawComponent::Math { opener: token, .. } => token.index,
            super::grammar::RawComponent::Authored { .. } => {
                return Err(ColorProtocolError::WrongSlot);
            }
        };
        if ordinal == 0 {
            if record.previous.is_some() {
                return Err(ColorProtocolError::WrongSlot);
            }
            if super::channels::legacy_rgb_domains(domains) {
                self.advance()
            } else {
                let ordinal = if domains[1] != domains[0] {
                    1
                } else if domains[2] != domains[0] {
                    2
                } else {
                    0
                };
                let site = sites[ordinal];
                self.state = State::Failed(SyntaxFailure::Component {
                    site,
                    reason: super::component::ComponentFailureReason::Component,
                });
                Ok(())
            }
        } else {
            let previous = record.previous.ok_or(ColorProtocolError::WrongSlot)?;
            if previous >= index {
                return Err(ColorProtocolError::WrongIndex);
            }
            self.state = State::LegacyRgb {
                index: previous,
                remaining: ordinal,
                domains,
                sites,
            };
            Ok(())
        }
    }
    pub(crate) fn component_read(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        let State::Component { cursor, .. } = &mut self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        cursor.read(bank, index, slot)
    }
    pub(crate) fn component_record(
        &self,
        bank: CssColorBank,
        index: usize,
    ) -> Result<CssColorSlot<'owner>, ColorProtocolError> {
        let State::Component { cursor, .. } = &self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        cursor.record(bank, index)
    }
    pub(crate) fn component_written(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        let State::Component { cursor, .. } = &mut self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        cursor.written(bank, index, slot)?;
        self.scratch_end = cursor.scratch_end();
        self.expression_end = cursor.expression_end();
        Ok(())
    }
    pub(crate) fn kernel(&self) -> crate::color::controlled::CssColorKernel {
        match &self.state {
            State::Component { cursor, .. } => cursor.kernel(),
            _ => crate::color::controlled::CssColorKernel::Transition,
        }
    }
    pub(crate) fn compute(&mut self) -> Result<(), ColorProtocolError> {
        let State::Component { record, cursor } = &mut self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        match cursor.action() {
            ComponentAction::Done => {
                let mut record = *record;
                record.checked = Some(cursor.finish()?.scalar());
                self.index
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                self.state = State::Write(record);
                Ok(())
            }
            ComponentAction::Failed(failure) => {
                self.state = State::Failed(failure);
                Ok(())
            }
            ComponentAction::Compute => cursor.compute(),
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    pub(crate) fn replacement(
        &self,
        index: usize,
    ) -> Result<CssColorSlot<'owner>, ColorProtocolError> {
        if self.action() != (SyntaxAction::Replace { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        let State::Write(record) = self.state else {
            unreachable!()
        };
        Ok(CssColorSlot::new(
            self.table.input.records().owner,
            self.table.input.records().phase,
            index,
            Data::Grammar(GrammarRecord::Component(record)),
        ))
    }
    pub(crate) fn replaced(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if self.action() != (SyntaxAction::Replace { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        let State::Write(expected) = self.state else {
            unreachable!()
        };
        let GrammarRecord::Component(actual) = self.table.record(index, slot)? else {
            return Err(ColorProtocolError::WrongSlot);
        };
        if expected != actual {
            return Err(ColorProtocolError::WrongSlot);
        }
        self.advance()
    }
    pub(crate) const fn scratch_end(&self) -> usize {
        self.scratch_end
    }
    pub(crate) fn finish(&self) -> Result<ValidatedGrammar<'owner>, ColorProtocolError> {
        if !matches!(self.state, State::Done) {
            return Err(ColorProtocolError::Terminal);
        }
        Ok(ValidatedGrammar {
            table: self.table,
            expression_end: self.expression_end,
        })
    }
    pub(crate) fn abandon(&mut self) {
        if let State::Component { cursor, .. } = &mut self.state {
            cursor.abandon();
        }
        self.state = State::Stopped;
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::super::component::CheckedComponent;
    use super::super::grammar::{ColorGrammar, GrammarAction};
    use super::*;
    use crate::color::controlled::CssColorOperationOwner;
    use crate::color::controlled::protocol::{ActionDescription, LexicalOperation};
    use surgeist_css_parser::input::InputKind;

    fn lexical<'owner>(
        owner: &'owner mut CssColorOperationOwner,
        input: &str,
    ) -> (
        crate::color::controlled::protocol::LexicalTable<'owner>,
        Vec<CssColorSlot<'owner>>,
    ) {
        let mut operation = LexicalOperation::start(owner, InputKind::Utf8, input.len()).unwrap();
        let mut slots = Vec::new();
        loop {
            let mut action = operation.next_action();
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
        (operation.finish().unwrap(), slots)
    }
    pub(crate) struct Fixture<'owner> {
        pub(crate) grammar: ValidatedGrammar<'owner>,
        pub(crate) slots: Vec<CssColorSlot<'owner>>,
        pub(crate) arithmetic: Vec<CssColorSlot<'owner>>,
        counts: (usize, usize),
    }
    pub(crate) fn fixture<'owner>(
        owner: &'owner mut CssColorOperationOwner,
        input: &str,
    ) -> Result<Fixture<'owner>, SyntaxFailure> {
        let (lexical, mut slots) = lexical(owner, input);
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
                GrammarAction::Failed { source } => panic!("fixture structure {:?}", source),
                GrammarAction::Stopped => panic!("grammar stop"),
            }
        }
        let table = grammar.finish().unwrap();
        let original: Vec<_> = (table.start..table.end)
            .map(|i| table.record(i, &slots[i]).unwrap())
            .collect();
        let mut pass = SyntaxPass::start(table, slots.len(), 0).unwrap();
        let mut arithmetic = Vec::new();
        let mut record_reads = 0;
        loop {
            match pass.action() {
                SyntaxAction::ReadRecord { index } => {
                    record_reads += 1;
                    pass.read_record(index, &slots[index]).unwrap();
                }
                SyntaxAction::ReadLegacyRgb { index } => {
                    pass.read_legacy_rgb(index, &slots[index]).unwrap()
                }
                SyntaxAction::Component(action) => match action {
                    ComponentAction::Read { bank, index } => {
                        let slot = match bank {
                            CssColorBank::Algorithm => &slots[index],
                            CssColorBank::Arithmetic => &arithmetic[index],
                            CssColorBank::Evaluation => panic!("syntax evaluation"),
                        };
                        pass.component_read(bank, index, slot).unwrap();
                    }
                    ComponentAction::Append { bank, index } => {
                        let destination = match bank {
                            CssColorBank::Algorithm => &mut slots,
                            CssColorBank::Arithmetic => &mut arithmetic,
                            CssColorBank::Evaluation => panic!("syntax evaluation"),
                        };
                        assert_eq!(index, destination.len());
                        destination.push(pass.component_record(bank, index).unwrap());
                        pass.component_written(bank, index, &destination[index])
                            .unwrap();
                    }
                    ComponentAction::Replace { bank, index } => {
                        assert_eq!(bank, CssColorBank::Algorithm);
                        slots[index] = pass.component_record(bank, index).unwrap();
                        pass.component_written(bank, index, &slots[index]).unwrap();
                    }
                    ComponentAction::Compute
                    | ComponentAction::Done
                    | ComponentAction::Failed(_) => pass.compute().unwrap(),
                    ComponentAction::Stopped => panic!("component stop"),
                },
                SyntaxAction::Replace { index } => {
                    assert_eq!(
                        pass.replaced(index, &slots[index]),
                        Err(ColorProtocolError::WrongSlot)
                    );
                    assert_eq!(pass.action(), SyntaxAction::Replace { index });
                    slots[index] = pass.replacement(index).unwrap();
                    pass.replaced(index, &slots[index]).unwrap();
                }
                SyntaxAction::Done => break,
                SyntaxAction::Failed(failure) => {
                    pass.abandon();
                    return Err(failure);
                }
                SyntaxAction::Stopped => panic!("pass stop"),
            }
        }
        assert_eq!(record_reads, table.end - table.start);
        assert_eq!(pass.scratch_end(), slots.len());
        let result = pass.finish().unwrap();
        assert_eq!(result.expression_end, arithmetic.len());
        assert_eq!(result.table.root, table.root);
        let mut checked = 0;
        let mut expressions = 0;
        for (index, old) in (table.start..table.end).zip(original) {
            let now = result.table.record(index, &slots[index]).unwrap();
            match (old, now) {
                (GrammarRecord::Component(old), GrammarRecord::Component(mut now)) => {
                    let scalar = now.checked.take().expect("every actual component checked");
                    checked += 1;
                    assert_eq!(old, now);
                    if let CheckedComponent::Expression {
                        table: expression, ..
                    } = scalar
                        .component(&crate::color::grammar::GrammarInput::Source(lexical))
                        .unwrap()
                    {
                        assert!(
                            expression.start < expression.length
                                && expression.length <= arithmetic.len()
                        );
                        expressions += 1;
                    }
                }
                (old, now) => assert_eq!(old, now),
            }
        }
        pass.abandon();
        assert_eq!(pass.action(), SyntaxAction::Stopped);
        assert_eq!(pass.finish().err(), Some(ColorProtocolError::Terminal));
        Ok(Fixture {
            grammar: result,
            slots,
            arithmetic,
            counts: (checked, expressions),
        })
    }
    fn check(input: &str) -> Result<(usize, usize), SyntaxFailure> {
        let mut owner = CssColorOperationOwner::new();
        fixture(&mut owner, input).map(|fixture| fixture.counts)
    }
    #[test]
    fn all_actual_component_records_are_checked_once_in_emission_order() {
        assert_eq!(
            check("light-dark(rgb(calc(1 + 2) 0 0 / .5), color(--profile 0 calc(2 * 3) none))"),
            Ok((7, 2))
        );
        assert_eq!(
            check("color-mix(in srgb, red 20%, rgb(from blue r calc(g * 2) b / alpha) calc(30%))"),
            Ok((6, 2))
        );
        assert_eq!(check("rgb(1 2 calc(3"), Ok((3, 1)));
    }
    #[test]
    fn legacy_rgb_relationship_and_unselected_syntax_do_not_reach_facts() {
        assert_eq!(check("rgb(1,2,3,.5)"), Ok((4, 0)));
        assert_eq!(check("rgb(1%,calc(2%),3%)"), Ok((3, 1)));
        assert!(check("rgb(1%,2,3%)").is_err());
        assert!(check("light-dark(red,rgb(calc(1em) 0 0))").is_err());
        assert!(check("rgb(from red calc(h) g b)").is_err());
        assert_eq!(
            check("color(from red --profile é calc(up * 2) 0)"),
            Ok((3, 1))
        );
    }
}
