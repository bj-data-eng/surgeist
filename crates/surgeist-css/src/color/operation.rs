//! One parse-only continuation over the canonical finite phase owners.
//!
//! Caller storage remains outside this machine. A reached error terminates the
//! operation without taking any bank; checkpoints contain no scratch index.

use super::component::{ComponentAction, SyntaxFailure};
use super::grammar::{ColorGrammar, GrammarAction, Site};
use super::resolution::{Action as ResolutionAction, Resolution};
use super::syntax_pass::{SyntaxAction, SyntaxPass, ValidatedGrammar};
use crate::color::controlled::protocol::ActionDescription as LexicalAction;
use crate::color::controlled::protocol::{
    ColorProtocolError, Data, LexicalOperation, LexicalTable, RecordTable,
};
use crate::color::controlled::{CssColorBank, CssColorOperationOwner, CssColorSlot};
use crate::color::{CssColorFactIdentity, CssColorFacts};
use crate::tokenization::bounded::InputKind;
use crate::tokenization::tokenizer::Span;
use surgeist_color::Color;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GrammarRequest {
    PrefixColor,
    WholeColor,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StandardReason {
    ColorGrammar,
    ComponentGrammar,
    NumericGrammar,
    InvalidProfile,
    InvalidComponents,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StandardFailure {
    pub(crate) reason: StandardReason,
    pub(crate) site: Site,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Protocol(ColorProtocolError),
    Resolution(super::resolution::Error),
}
impl From<ColorProtocolError> for Error {
    fn from(error: ColorProtocolError) -> Self {
        Self::Protocol(error)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Action {
    Lexical(LexicalAction),
    Grammar(GrammarAction),
    Syntax(SyntaxAction),
    Resolution(ResolutionAction),
    PrepareGrammar,
    PrepareSyntax,
    PrepareResolution,
    FailureSource { index: usize },
    UsedReady,
    StandardFailed,
    Stopped,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StorageAction {
    Read { bank: CssColorBank, index: usize },
    Append { bank: CssColorBank, index: usize },
    Replace { bank: CssColorBank, index: usize },
}
impl Action {
    pub(crate) fn storage(self) -> Option<StorageAction> {
        use super::profile::{ComponentNameAction, ProfileAction};
        use super::profile_conversion::{ConversionAction, InputAction};
        let read = |bank, index| Some(StorageAction::Read { bank, index });
        let append = |bank, index| Some(StorageAction::Append { bank, index });
        let replace = |bank, index| Some(StorageAction::Replace { bank, index });
        match self {
            Self::Lexical(LexicalAction::ReadSlot { index, .. })
            | Self::Grammar(
                GrammarAction::ReadLexical { index }
                | GrammarAction::ReadFrame { index }
                | GrammarAction::ReadHex { index },
            )
            | Self::Syntax(
                SyntaxAction::ReadRecord { index } | SyntaxAction::ReadLegacyRgb { index },
            )
            | Self::FailureSource { index } => read(CssColorBank::Algorithm, index),
            Self::Lexical(LexicalAction::AppendSlot { index, .. })
            | Self::Grammar(GrammarAction::Append { index }) => {
                append(CssColorBank::Algorithm, index)
            }
            Self::Syntax(SyntaxAction::Replace { index }) => {
                replace(CssColorBank::Algorithm, index)
            }
            Self::Syntax(SyntaxAction::Component(action)) => match action {
                ComponentAction::Read { bank, index } => read(bank, index),
                ComponentAction::Append { bank, index } => append(bank, index),
                ComponentAction::Replace { bank, index } => replace(bank, index),
                _ => None,
            },
            Self::Resolution(action) => match action {
                ResolutionAction::ReadRecipe { index }
                | ResolutionAction::ReadParent { index }
                | ResolutionAction::ReadMix { index }
                | ResolutionAction::ReadTask { index }
                | ResolutionAction::ReadProfileToken { index }
                | ResolutionAction::ReadProfileNameToken { index }
                | ResolutionAction::Profile(
                    ProfileAction::ReadPrefix { index } | ProfileAction::ReadSource { index },
                )
                | ResolutionAction::ProfileName(ComponentNameAction::ReadSource { index })
                | ResolutionAction::Conversion(ConversionAction::ReadInput { index, .. }) => {
                    read(CssColorBank::Algorithm, index)
                }
                ResolutionAction::AppendFrame { index }
                | ResolutionAction::AppendTask { index }
                | ResolutionAction::ProfileInput(InputAction::Append { index }) => {
                    append(CssColorBank::Algorithm, index)
                }
                ResolutionAction::ReplaceTask { index } => replace(CssColorBank::Algorithm, index),
                ResolutionAction::Component(action) => match action {
                    super::component_value::Action::ReadComponent { index } => {
                        read(CssColorBank::Algorithm, index)
                    }
                    super::component_value::Action::Read { bank, index } => read(bank, index),
                    super::component_value::Action::Append { index } => {
                        append(CssColorBank::Evaluation, index)
                    }
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }
}
#[expect(
    clippy::large_enum_variant,
    reason = "Lexical, grammar, syntax and resolution are mutually exclusive phases; the advertised inline receiver layout is caller-accounted without hidden heap allocation"
)]
enum Phase<'owner, 'facts> {
    Lexical(LexicalOperation<'owner>),
    Grammar(ColorGrammar<'owner>),
    Syntax(SyntaxPass<'owner>),
    Resolution(Resolution<'owner, 'facts>),
    FailureSource {
        index: usize,
        reason: StandardReason,
    },
    Used {
        value: Color,
        consumed: Site,
    },
    Standard(StandardFailure),
    Stopped,
}
/// Source and fact qualification survives scratch retirement; no bank link or
/// owner borrow does, so the owner can serve another parse.
pub(crate) struct CheckpointContext<'facts> {
    pub(crate) facts: &'facts CssColorFacts<'facts>,
    pub(crate) expected: CssColorFactIdentity,
    pub(crate) input_kind: InputKind,
    pub(crate) input_length: usize,
    pub(crate) request: GrammarRequest,
    pub(crate) consumed: Span,
}
pub(crate) enum ParseOutcome<'facts> {
    Used(UsedCheckpoint<'facts>),
    StandardFailure(FailureCheckpoint<'facts>),
}
pub(crate) struct UsedCheckpoint<'facts> {
    pub(crate) context: CheckpointContext<'facts>,
    pub(crate) value: Color,
    pub(crate) standard_failure: Option<StandardFailure>,
}
pub(crate) struct FailureCheckpoint<'facts> {
    context: CheckpointContext<'facts>,
    failure: StandardFailure,
}
impl<'facts> UsedCheckpoint<'facts> {
    pub(crate) const fn value(&self) -> Color {
        self.value
    }
    pub(crate) const fn context(&self) -> &CheckpointContext<'facts> {
        &self.context
    }
    pub(crate) const fn standard_failure(&self) -> Option<StandardFailure> {
        self.standard_failure
    }
}
impl<'facts> FailureCheckpoint<'facts> {
    pub(crate) const fn failure(&self) -> StandardFailure {
        self.failure
    }
    pub(crate) fn fallback(self, value: Color) -> UsedCheckpoint<'facts> {
        UsedCheckpoint {
            context: self.context,
            value,
            standard_failure: Some(self.failure),
        }
    }
}
struct SourceContext<'css> {
    lexical: Option<LexicalTable<'css>>,
    kind: InputKind,
    length: usize,
    request: GrammarRequest,
}
enum InputContext<'css> {
    Source(SourceContext<'css>),
    Authored(RecordTable<'css>),
}
impl<'css> InputContext<'css> {
    fn records(&self) -> Option<RecordTable<'css>> {
        match self {
            Self::Source(source) => source.lexical.map(|table| table.records()),
            Self::Authored(records) => Some(*records),
        }
    }
    fn source(&self) -> Result<&SourceContext<'css>, ColorProtocolError> {
        match self {
            Self::Source(source) => Ok(source),
            Self::Authored(_) => Err(ColorProtocolError::WrongSlot),
        }
    }
}
pub(crate) struct ColorOperation<'owner, 'facts> {
    input: InputContext<'owner>,
    phase: Phase<'owner, 'facts>,
    facts: &'facts CssColorFacts<'facts>,
    expected: CssColorFactIdentity,
    banks: [usize; 3],
}
fn ordinal(bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => 0,
        CssColorBank::Arithmetic => 1,
        CssColorBank::Evaluation => 2,
    }
}
impl<'owner, 'facts> ColorOperation<'owner, 'facts> {
    pub(crate) fn start(
        owner: &'owner mut CssColorOperationOwner,
        kind: InputKind,
        length: usize,
        request: GrammarRequest,
        facts: &'facts CssColorFacts<'facts>,
        expected: CssColorFactIdentity,
    ) -> Result<Self, ColorProtocolError> {
        let operation = LexicalOperation::start(owner, kind, length)?;
        Ok(Self {
            input: InputContext::Source(SourceContext {
                lexical: None,
                kind,
                length,
                request,
            }),
            phase: Phase::Lexical(operation),
            facts,
            expected,
            banks: [0; 3],
        })
    }
    pub(crate) fn start_authored(
        grammar: &ValidatedGrammar<'owner>,
        facts: &'facts CssColorFacts<'facts>,
        expected: CssColorFactIdentity,
        banks: [usize; 3],
    ) -> Result<Self, ColorProtocolError> {
        let super::grammar::GrammarInput::Authored(records) = grammar.table.input else {
            return Err(ColorProtocolError::WrongSlot);
        };
        if grammar.table.end > banks[0]
            || records.length > banks[0]
            || grammar.expression_end != banks[1]
        {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(Self {
            input: InputContext::Authored(records),
            phase: Phase::Resolution(Resolution::start(grammar, facts, expected)),
            facts,
            expected,
            banks,
        })
    }
    pub(crate) fn reached_site(&self) -> Result<Site, ColorProtocolError> {
        match &self.phase {
            Phase::Resolution(cursor) => cursor.reached_site(),
            Phase::Standard(failure) => Ok(failure.site),
            Phase::Used { consumed, .. } => Ok(*consumed),
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    pub(crate) fn take_authored(
        self,
    ) -> Result<Result<Color, StandardFailure>, ColorProtocolError> {
        if !matches!(self.input, InputContext::Authored(_)) {
            return Err(ColorProtocolError::WrongSlot);
        }
        match self.phase {
            Phase::Used { value, .. } => Ok(Ok(value)),
            Phase::Standard(failure) => Ok(Err(failure)),
            _ => Err(ColorProtocolError::Terminal),
        }
    }
    /// The anchor and phase that qualify every record this parse mints.
    pub(crate) fn qualifier(&self) -> Option<(&'owner CssColorOperationOwner, usize)> {
        match &self.phase {
            Phase::Lexical(cursor) => Some(cursor.qualifier()),
            _ => self.input.records().map(|table| (table.owner, table.phase)),
        }
    }
    pub(crate) fn action(&self) -> Action {
        match &self.phase {
            Phase::Lexical(cursor) => match cursor.description() {
                LexicalAction::Done => Action::PrepareGrammar,
                action => Action::Lexical(action),
            },
            Phase::Grammar(cursor) => match cursor.action() {
                GrammarAction::Done { .. } => Action::PrepareSyntax,
                action => Action::Grammar(action),
            },
            Phase::Syntax(cursor) => match cursor.action() {
                SyntaxAction::Done => Action::PrepareResolution,
                action => Action::Syntax(action),
            },
            Phase::Resolution(cursor) => Action::Resolution(cursor.action()),
            Phase::FailureSource { index, .. } => Action::FailureSource { index: *index },
            Phase::Used { .. } => Action::UsedReady,
            Phase::Standard(_) => Action::StandardFailed,
            Phase::Stopped => Action::Stopped,
        }
    }
    pub(crate) const fn bank_length(&self, bank: CssColorBank) -> usize {
        self.banks[match bank {
            CssColorBank::Algorithm => 0,
            CssColorBank::Arithmetic => 1,
            CssColorBank::Evaluation => 2,
        }]
    }
    pub(crate) fn read_input(&mut self, offset: usize, unit: u16) -> Result<(), Error> {
        let Phase::Lexical(cursor) = &mut self.phase else {
            return Err(ColorProtocolError::WrongAction.into());
        };
        cursor.next_action().read_input(offset, unit)?;
        Ok(())
    }
    pub(crate) fn read_slot(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        if index >= self.bank_length(bank) {
            return Err(ColorProtocolError::WrongIndex.into());
        }
        if let Some(table) = self.input.records() {
            slot.qualify(table.owner, table.phase, index, bank)?;
        }
        let action = self.action();
        match (&mut self.phase, action) {
            (
                Phase::Lexical(cursor),
                Action::Lexical(LexicalAction::ReadSlot {
                    index: expected, ..
                }),
            ) if index == expected && bank == CssColorBank::Algorithm => {
                cursor.next_action().read_slot(index, slot)?
            }
            (
                Phase::Grammar(cursor),
                Action::Grammar(
                    GrammarAction::ReadLexical { index: expected }
                    | GrammarAction::ReadFrame { index: expected }
                    | GrammarAction::ReadHex { index: expected },
                ),
            ) if bank == CssColorBank::Algorithm && index == expected => match cursor.action() {
                GrammarAction::ReadLexical { .. } => cursor.read_lexical(index, slot)?,
                GrammarAction::ReadFrame { .. } => cursor.read_frame(index, slot)?,
                GrammarAction::ReadHex { .. } => cursor.read_hex(index, slot)?,
                _ => unreachable!(),
            },
            (
                Phase::Syntax(cursor),
                Action::Syntax(SyntaxAction::ReadRecord { index: expected }),
            ) if bank == CssColorBank::Algorithm && index == expected => {
                cursor.read_record(index, slot)?
            }
            (
                Phase::Syntax(cursor),
                Action::Syntax(SyntaxAction::ReadLegacyRgb { index: expected }),
            ) if bank == CssColorBank::Algorithm && index == expected => {
                cursor.read_legacy_rgb(index, slot)?
            }
            (
                Phase::Syntax(cursor),
                Action::Syntax(SyntaxAction::Component(ComponentAction::Read {
                    bank: expected_bank,
                    index: expected,
                })),
            ) if bank == expected_bank && index == expected => {
                cursor.component_read(bank, index, slot)?
            }
            (Phase::Resolution(cursor), Action::Resolution(action)) => {
                use super::profile::{ComponentNameAction, ProfileAction};
                use super::profile_conversion::ConversionAction;
                let result = match action {
                    ResolutionAction::ReadRecipe { index: expected }
                        if bank == CssColorBank::Algorithm && index == expected =>
                    {
                        cursor.read_recipe(index, slot)
                    }
                    ResolutionAction::ReadParent { index: expected }
                        if bank == CssColorBank::Algorithm && index == expected =>
                    {
                        cursor.read_parent(index, slot)
                    }
                    ResolutionAction::ReadMix { index: expected }
                        if bank == CssColorBank::Algorithm && index == expected =>
                    {
                        cursor.read_mix(index, slot, self.banks[0])
                    }
                    ResolutionAction::ReadTask { index: expected }
                        if bank == CssColorBank::Algorithm && index == expected =>
                    {
                        cursor.read_task(index, slot, self.banks[2])
                    }
                    ResolutionAction::Component(
                        super::component_value::Action::ReadComponent { index: expected },
                    ) if bank == CssColorBank::Algorithm && index == expected => {
                        cursor.read_component(index, slot)
                    }
                    ResolutionAction::Component(super::component_value::Action::Read {
                        bank: expected_bank,
                        index: expected,
                    }) if bank == expected_bank && index == expected => {
                        cursor.component_read(bank, index, slot)
                    }
                    ResolutionAction::ReadProfileToken { index: expected }
                        if bank == CssColorBank::Algorithm && index == expected =>
                    {
                        cursor.read_profile_token(index, slot)
                    }
                    ResolutionAction::Profile(
                        ProfileAction::ReadPrefix { index: expected }
                        | ProfileAction::ReadSource { index: expected },
                    )
                    | ResolutionAction::ProfileName(ComponentNameAction::ReadSource {
                        index: expected,
                    }) if bank == CssColorBank::Algorithm && index == expected => {
                        cursor.profile_source(index, slot)
                    }
                    ResolutionAction::ReadProfileNameToken { index: expected }
                        if bank == CssColorBank::Algorithm && index == expected =>
                    {
                        cursor.read_profile_name_token(index, slot)
                    }
                    ResolutionAction::Conversion(ConversionAction::ReadInput {
                        index: expected,
                        ordinal,
                    }) if bank == CssColorBank::Algorithm && index == expected => {
                        cursor.conversion_input(index, ordinal, slot)
                    }
                    _ => return Err(ColorProtocolError::WrongAction.into()),
                };
                if let Err(error) = result {
                    return self.resolution_error(error);
                }
            }
            (
                Phase::FailureSource {
                    index: expected,
                    reason,
                },
                Action::FailureSource { .. },
            ) if bank == CssColorBank::Algorithm && index == *expected => {
                let table = self
                    .input
                    .source()?
                    .lexical
                    .ok_or(ColorProtocolError::WrongAction)?;
                let crate::color::controlled::protocol::LexicalRecord::Token(token) =
                    table.observe(index, slot)?
                else {
                    return Err(ColorProtocolError::WrongSlot.into());
                };
                self.phase = Phase::Standard(StandardFailure {
                    reason: *reason,
                    site: Site::Source(token.source),
                });
            }
            _ => return Err(ColorProtocolError::WrongAction.into()),
        }
        Ok(())
    }
    pub(crate) fn pending_record(
        &mut self,
        bank: CssColorBank,
        index: usize,
    ) -> Result<CssColorSlot<'owner>, Error> {
        let action = self.action();
        match (&mut self.phase, action) {
            (
                Phase::Lexical(cursor),
                Action::Lexical(LexicalAction::AppendSlot {
                    index: expected, ..
                }),
            ) if bank == CssColorBank::Algorithm && index == expected => {
                Ok(cursor.next_action().append_record()?)
            }
            (
                Phase::Grammar(cursor),
                Action::Grammar(GrammarAction::Append { index: expected }),
            ) if bank == CssColorBank::Algorithm && index == expected => Ok(cursor.record(index)?),
            (Phase::Syntax(cursor), Action::Syntax(SyntaxAction::Replace { index: expected }))
                if bank == CssColorBank::Algorithm && index == expected =>
            {
                Ok(cursor.replacement(index)?)
            }
            (
                Phase::Syntax(cursor),
                Action::Syntax(SyntaxAction::Component(
                    ComponentAction::Append {
                        bank: expected_bank,
                        index: expected,
                    }
                    | ComponentAction::Replace {
                        bank: expected_bank,
                        index: expected,
                    },
                )),
            ) if bank == expected_bank && index == expected => {
                Ok(cursor.component_record(bank, index)?)
            }
            (Phase::Resolution(cursor), Action::Resolution(action)) => {
                use super::profile_conversion::InputAction;
                let result = match action {
                    ResolutionAction::AppendFrame { index: expected }
                    | ResolutionAction::AppendTask { index: expected }
                    | ResolutionAction::ReplaceTask { index: expected }
                        if bank == CssColorBank::Algorithm && index == expected =>
                    {
                        cursor.record(index)
                    }
                    ResolutionAction::ProfileInput(InputAction::Append { index: expected })
                        if bank == CssColorBank::Algorithm && index == expected =>
                    {
                        cursor.profile_input_record(index)
                    }
                    ResolutionAction::Component(super::component_value::Action::Append {
                        index: expected,
                    }) if bank == CssColorBank::Evaluation && index == expected => {
                        cursor.component_record(index)
                    }
                    _ => return Err(ColorProtocolError::WrongAction.into()),
                };
                result.map_err(Error::Resolution)
            }
            _ => Err(ColorProtocolError::WrongAction.into()),
        }
    }
    pub(crate) fn written(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        let action = self.action();
        let replacing = matches!(
            action,
            Action::Syntax(
                SyntaxAction::Replace { .. }
                    | SyntaxAction::Component(ComponentAction::Replace { .. })
            ) | Action::Resolution(ResolutionAction::ReplaceTask { .. })
        );
        let length = self.bank_length(bank);
        if replacing && index >= length || !replacing && index != length {
            return Err(ColorProtocolError::WrongIndex.into());
        }
        let next = if replacing {
            length
        } else {
            length.checked_add(1).ok_or(ColorProtocolError::Overflow)?
        };
        if let Some(table) = self.input.records() {
            slot.qualify(table.owner, table.phase, index, bank)?;
        }
        match (&mut self.phase, action) {
            (
                Phase::Lexical(cursor),
                Action::Lexical(LexicalAction::AppendSlot {
                    index: expected, ..
                }),
            ) if bank == CssColorBank::Algorithm && index == expected => {
                let mut guard = cursor.next_action();
                let pending = guard.append_record()?;
                if !same_lexical(&pending, slot) {
                    return Err(ColorProtocolError::WrongSlot.into());
                }
                guard.appended(index)?;
            }
            (
                Phase::Grammar(cursor),
                Action::Grammar(GrammarAction::Append { index: expected }),
            ) if bank == CssColorBank::Algorithm && index == expected => {
                cursor.appended(index, slot)?
            }
            (Phase::Syntax(cursor), Action::Syntax(SyntaxAction::Replace { index: expected }))
                if bank == CssColorBank::Algorithm && index == expected =>
            {
                cursor.replaced(index, slot)?
            }
            (
                Phase::Syntax(cursor),
                Action::Syntax(SyntaxAction::Component(
                    ComponentAction::Append {
                        bank: expected_bank,
                        index: expected,
                    }
                    | ComponentAction::Replace {
                        bank: expected_bank,
                        index: expected,
                    },
                )),
            ) if bank == expected_bank && index == expected => {
                cursor.component_written(bank, index, slot)?
            }
            (Phase::Resolution(cursor), Action::Resolution(action)) => {
                use super::profile_conversion::InputAction;
                let result = match action {
                    ResolutionAction::AppendFrame { index: expected }
                    | ResolutionAction::AppendTask { index: expected }
                    | ResolutionAction::ReplaceTask { index: expected }
                        if bank == CssColorBank::Algorithm && index == expected =>
                    {
                        cursor.written(index, slot)
                    }
                    ResolutionAction::ProfileInput(InputAction::Append { index: expected })
                        if bank == CssColorBank::Algorithm && index == expected =>
                    {
                        cursor.profile_input_written(index, slot)
                    }
                    ResolutionAction::Component(super::component_value::Action::Append {
                        index: expected,
                    }) if bank == CssColorBank::Evaluation && index == expected => {
                        cursor.component_written(index, slot)
                    }
                    _ => return Err(ColorProtocolError::WrongAction.into()),
                };
                if let Err(error) = result {
                    return self.resolution_error(error);
                }
            }
            _ => return Err(ColorProtocolError::WrongAction.into()),
        }
        self.banks[ordinal(bank)] = next;
        Ok(())
    }
    fn standard_source(&mut self, reason: StandardReason, site: usize) -> Result<(), Error> {
        let table = self
            .input
            .source()?
            .lexical
            .ok_or(ColorProtocolError::WrongAction)?;
        self.abandon();
        self.phase = if site == table.length() {
            Phase::Standard(StandardFailure {
                reason,
                site: Site::Source(Span {
                    start: self.input.source()?.length,
                    end: self.input.source()?.length,
                }),
            })
        } else if site < table.length() {
            Phase::FailureSource {
                index: site,
                reason,
            }
        } else {
            return Err(ColorProtocolError::WrongIndex.into());
        };
        Ok(())
    }
    fn resolution_error(&mut self, error: super::resolution::Error) -> Result<(), Error> {
        use super::profile_conversion::Error as C;
        use super::resolution::Error as R;
        // Exhausting the mix argument counter is not a reachable input; it
        // stops the operation as a counter overflow.
        if let R::Interpolation(super::interpolate::Error::Overflow) = error {
            self.abandon();
            return Err(ColorProtocolError::Overflow.into());
        }
        // Only an actual standard-invalid branch can mint a fallback checkpoint.
        let reason = match error {
            R::Conversion(C::InvalidColor) => Some(StandardReason::InvalidProfile),
            R::Channel(super::channels::Error::Grammar)
            | R::Interpolation(super::interpolate::Error::InvalidWeight) => {
                Some(StandardReason::InvalidComponents)
            }
            R::ProfileComponent(
                super::profile::ComponentNameError::UnknownChannel
                | super::profile::ComponentNameError::InvalidProfile,
            ) => Some(StandardReason::InvalidProfile),
            _ => None,
        };
        if let R::Protocol(error) = error {
            return Err(Error::Protocol(error));
        }
        let site = if reason.is_some() {
            Some(match &self.input {
                InputContext::Source(source) => Site::Source(Span {
                    start: 0,
                    end: source.length,
                }),
                InputContext::Authored(_) => {
                    let Phase::Resolution(cursor) = &self.phase else {
                        return Err(ColorProtocolError::WrongAction.into());
                    };
                    cursor.reached_site()?
                }
            })
        } else {
            None
        };
        self.abandon();
        if let Some(reason) = reason {
            self.phase = Phase::Standard(StandardFailure {
                reason,
                site: site.expect("standard failure site was prepared above"),
            });
            Ok(())
        } else {
            Err(Error::Resolution(error))
        }
    }
    pub(crate) fn additional_fact_read(
        &self,
    ) -> Option<crate::color::controlled::CssColorFactRead> {
        match &self.phase {
            Phase::Resolution(cursor) => cursor.additional_fact_read(),
            _ => None,
        }
    }
    pub(crate) fn input_kind(&self) -> InputKind {
        self.input.source().expect("source action front door").kind
    }
    pub(crate) fn kernel(&self) -> crate::color::controlled::CssColorKernel {
        use crate::color::controlled::CssColorKernel as K;
        match &self.phase {
            Phase::Lexical(cursor) => match cursor.description() {
                LexicalAction::ComputeInput => K::Preprocess,
                LexicalAction::ComputeToken => K::Tokenize,
                _ => K::Transition,
            },
            Phase::Grammar(_) => K::Grammar,
            Phase::Syntax(cursor) => cursor.kernel(),
            Phase::Resolution(cursor) => cursor.kernel(),
            _ => K::Transition,
        }
    }
    pub(crate) fn compute(&mut self) -> Result<(), Error> {
        let action = self.action();
        match action {
            Action::PrepareGrammar => {
                let phase = std::mem::replace(&mut self.phase, Phase::Stopped);
                let Phase::Lexical(cursor) = phase else {
                    unreachable!()
                };
                let table = cursor.finish()?;
                let InputContext::Source(source) = &mut self.input else {
                    return Err(ColorProtocolError::WrongSlot.into());
                };
                source.lexical = Some(table);
                self.phase = Phase::Grammar(ColorGrammar::start(
                    &table,
                    self.input.source()?.request == GrammarRequest::WholeColor,
                ));
            }
            Action::PrepareSyntax => {
                let Phase::Grammar(cursor) = &self.phase else {
                    unreachable!()
                };
                let table = cursor.finish()?;
                self.phase = Phase::Syntax(SyntaxPass::start(table, self.banks[0], self.banks[1])?);
            }
            Action::PrepareResolution => {
                let Phase::Syntax(cursor) = &self.phase else {
                    unreachable!()
                };
                let grammar = cursor.finish()?;
                if cursor.scratch_end() != self.banks[0] || grammar.expression_end != self.banks[1]
                {
                    return Err(ColorProtocolError::WrongIndex.into());
                }
                self.phase =
                    Phase::Resolution(Resolution::start(&grammar, self.facts, self.expected));
            }
            Action::Lexical(LexicalAction::ComputeInput | LexicalAction::ComputeToken) => {
                let Phase::Lexical(cursor) = &mut self.phase else {
                    unreachable!()
                };
                cursor.next_action().compute()?;
            }
            Action::Grammar(GrammarAction::Failed { source }) => {
                self.abandon();
                self.phase = Phase::Standard(StandardFailure {
                    reason: StandardReason::ColorGrammar,
                    site: Site::Source(source),
                });
            }
            Action::Grammar(GrammarAction::Compute) => {
                let Phase::Grammar(cursor) = &mut self.phase else {
                    unreachable!()
                };
                cursor.compute()?;
            }
            Action::Syntax(SyntaxAction::Failed(failure)) => {
                let (reason, site) = match failure {
                    SyntaxFailure::Component { site, .. } => {
                        (StandardReason::ComponentGrammar, site)
                    }
                    SyntaxFailure::Numeric { site, .. } => (StandardReason::NumericGrammar, site),
                };
                self.standard_source(reason, site)?;
            }
            Action::Syntax(SyntaxAction::Component(
                ComponentAction::Compute | ComponentAction::Done | ComponentAction::Failed(_),
            )) => {
                let Phase::Syntax(cursor) = &mut self.phase else {
                    unreachable!()
                };
                cursor.compute()?;
            }
            Action::Resolution(ResolutionAction::ReadScheme) => {
                let Phase::Resolution(cursor) = &mut self.phase else {
                    unreachable!()
                };
                if let Err(error) = cursor.read_scheme() {
                    self.resolution_error(error)?;
                }
            }
            Action::Resolution(ResolutionAction::Done) => {
                let Phase::Resolution(cursor) = &self.phase else {
                    unreachable!()
                };
                let consumed = match &self.input {
                    InputContext::Source(source) => Site::Source(Span {
                        start: 0,
                        end: if source.request == GrammarRequest::WholeColor {
                            source.length
                        } else {
                            cursor.consumed_source()?.end
                        },
                    }),
                    InputContext::Authored(_) => cursor.consumed_site()?,
                };
                self.phase = Phase::Used {
                    value: cursor.finish()?,
                    consumed,
                };
            }
            Action::Resolution(_) => {
                let Phase::Resolution(cursor) = &mut self.phase else {
                    unreachable!()
                };
                if let Err(error) = cursor.compute(self.banks[0], self.banks[2]) {
                    self.resolution_error(error)?;
                }
            }
            _ => return Err(ColorProtocolError::WrongAction.into()),
        }
        Ok(())
    }
    pub(crate) fn take(self) -> Result<ParseOutcome<'facts>, ColorProtocolError> {
        let source = self.input.source()?;
        let mut context = CheckpointContext {
            facts: self.facts,
            expected: self.expected,
            input_kind: source.kind,
            input_length: source.length,
            request: source.request,
            consumed: Span {
                start: 0,
                end: source.length,
            },
        };
        match self.phase {
            Phase::Used { value, consumed } => {
                context.consumed = consumed.source()?;
                Ok(ParseOutcome::Used(UsedCheckpoint {
                    context,
                    value,
                    standard_failure: None,
                }))
            }
            Phase::Standard(failure) => Ok(ParseOutcome::StandardFailure(FailureCheckpoint {
                context,
                failure,
            })),
            _ => Err(ColorProtocolError::Terminal),
        }
    }
    pub(crate) fn abandon(&mut self) {
        match &mut self.phase {
            Phase::Lexical(cursor) => cursor.abandon(),
            Phase::Grammar(cursor) => cursor.abandon(),
            Phase::Syntax(cursor) => cursor.abandon(),
            Phase::Resolution(cursor) => cursor.abandon(),
            _ => {}
        }
        self.phase = Phase::Stopped;
    }
}
fn same_lexical(left: &CssColorSlot<'_>, right: &CssColorSlot<'_>) -> bool {
    if right
        .qualify(left.owner, left.phase, left.index, CssColorBank::Algorithm)
        .is_err()
    {
        return false;
    }
    match (left.data, right.data) {
        (Data::Scalar(a), Data::Scalar(b)) => a == b,
        (Data::NameUnit(a), Data::NameUnit(b)) => a == b,
        (
            Data::NumberDigit {
                exponent: a,
                value: av,
            },
            Data::NumberDigit {
                exponent: b,
                value: bv,
            },
        ) => a == b && av == bv,
        (Data::Token(a), Data::Token(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::controlled::CssUsedColorCheckpoint;
    use crate::color::{CssColorPropertyFacts, CssColorSchemeFact, CssDeviceColorFact};
    use surgeist_color::{
        SerializationAction as NumericalAction, SerializationRequest, SerializationStep,
        Transformation,
    };
    fn facts() -> CssColorFacts<'static> {
        CssColorFacts::new(
            CssColorFactIdentity::new(3, 5),
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Light,
            &[],
            &[],
            &[],
            CssDeviceColorFact::Absent,
        )
    }
    fn parse<'facts>(
        owner: &mut CssColorOperationOwner,
        input: &[u16],
        kind: InputKind,
        request: GrammarRequest,
        facts: &'facts CssColorFacts<'facts>,
    ) -> Result<ParseOutcome<'facts>, Error> {
        let mut cursor =
            ColorOperation::start(owner, kind, input.len(), request, facts, facts.identity)
                .unwrap();
        let mut banks = [Vec::new(), Vec::new(), Vec::new()];
        loop {
            let action = cursor.action();
            let result = if let Some(storage) = action.storage() {
                match storage {
                    StorageAction::Read { bank, index } => {
                        cursor.read_slot(bank, index, &banks[ordinal(bank)][index])
                    }
                    StorageAction::Append { bank, index } => {
                        let destination = &mut banks[ordinal(bank)];
                        assert_eq!(index, destination.len());
                        destination.push(cursor.pending_record(bank, index).unwrap());
                        cursor.written(bank, index, &destination[index])
                    }
                    StorageAction::Replace { bank, index } => {
                        let destination = &mut banks[ordinal(bank)];
                        destination[index] = cursor.pending_record(bank, index).unwrap();
                        cursor.written(bank, index, &destination[index])
                    }
                }
            } else {
                match action {
                    Action::Lexical(LexicalAction::ReadInput { offset }) => {
                        cursor.read_input(offset, input[offset])
                    }
                    Action::UsedReady | Action::StandardFailed => break,
                    Action::Stopped => panic!("unexpected stopped parse"),
                    _ => cursor.compute(),
                }
            };
            if let Err(error) = result {
                cursor.abandon();
                assert_eq!(cursor.action(), Action::Stopped);
                banks.iter_mut().for_each(Vec::clear);
                return Err(error);
            }
            for bank in [
                CssColorBank::Algorithm,
                CssColorBank::Arithmetic,
                CssColorBank::Evaluation,
            ] {
                assert_eq!(cursor.bank_length(bank), banks[ordinal(bank)].len());
            }
        }
        let result = cursor.take().unwrap();
        // This is ordinary test ownership. Dropping these real vectors proves
        // checkpoint independence, not the later HTML allocation ledger.
        banks.iter_mut().for_each(Vec::clear);
        drop(banks);
        Ok(result)
    }
    fn units(input: &str) -> Vec<u16> {
        input
            .as_bytes()
            .iter()
            .map(|byte| u16::from(*byte))
            .collect()
    }
    /// Serializes the checkpoint's value with Color's serialization, refusing
    /// the emission beyond `cap` output bytes as a consumer's admission would.
    fn serialize(
        checkpoint: UsedCheckpoint<'_>,
        cap: usize,
    ) -> Result<(String, usize, Option<StandardFailure>), usize> {
        let failure = checkpoint.standard_failure();
        let mut numerical_owner = surgeist_color::OperationOwner::new();
        let mut serialization = surgeist_color::Serialization::new(
            &mut numerical_owner,
            checkpoint.value(),
            SerializationRequest::HtmlCompatible,
        );
        let mut digits = Vec::new();
        let mut output = Vec::new();
        let receipt = loop {
            let action = match serialization.step() {
                SerializationStep::Complete(receipt) => break receipt,
                SerializationStep::Pending(action) => action,
            };
            serialization = match action {
                NumericalAction::Compute(compute) => compute.compute(),
                NumericalAction::ReadDigit(read) => {
                    let slot = &digits[read.index()];
                    read.read(slot)
                }
                NumericalAction::AppendDigit(write) => {
                    assert_eq!(write.index(), digits.len());
                    let (record, receipt) = write.record();
                    digits.push(record);
                    receipt.stored(&digits[digits.len() - 1])
                }
                NumericalAction::ReplaceDigit(write) => {
                    let index = write.index();
                    let (record, receipt) = write.record();
                    digits[index] = record;
                    receipt.stored(&digits[index])
                }
                NumericalAction::Emit(emit) => {
                    if output.len() == cap {
                        // Refused admission abandons the serialization.
                        digits.clear();
                        return Err(output.len());
                    }
                    output.push(emit.unit());
                    emit.emitted()
                }
            };
        };
        assert_eq!(receipt.value().alpha(), Some(1.0));
        Ok((
            String::from_utf8(output).unwrap(),
            receipt.length(),
            failure,
        ))
    }
    fn transform<'facts>(
        checkpoint: UsedCheckpoint<'facts>,
        effect: Transformation,
    ) -> UsedCheckpoint<'facts> {
        let mut numerical_owner = surgeist_color::OperationOwner::new();
        CssUsedColorCheckpoint { checkpoint }
            .transform(&mut numerical_owner, effect)
            .compute()
            .checkpoint
    }
    #[test]
    fn parse_only_checkpoint_precedes_transform_and_final_output_admission() {
        let facts = facts();
        for cap in [6, 7] {
            let mut owner = CssColorOperationOwner::new();
            let ParseOutcome::Used(checkpoint) = parse(
                &mut owner,
                &units("rgb(255.000000000000000000000 0 0 / .2)"),
                InputKind::Utf8,
                GrammarRequest::WholeColor,
                &facts,
            )
            .unwrap() else {
                panic!("valid used")
            };
            assert_eq!(
                checkpoint.value().channels(),
                [Some(1.0), Some(0.0), Some(0.0)]
            );
            assert_eq!(checkpoint.value().alpha(), Some(0.2));
            assert_eq!(checkpoint.standard_failure(), None);
            assert_eq!(checkpoint.context().request, GrammarRequest::WholeColor);
            let checkpoint = transform(checkpoint, Transformation::ForceOpaqueAlpha);
            let checkpoint = transform(
                checkpoint,
                Transformation::Convert(surgeist_color::ColorSpace::Predefined(
                    surgeist_color::PredefinedColorSpace::Srgb,
                )),
            );
            let checkpoint = transform(checkpoint, Transformation::QuantizeSrgb);
            assert_eq!(
                serialize(checkpoint, cap),
                if cap == 6 {
                    Err(6)
                } else {
                    Ok(("#ff0000".to_owned(), 7, None))
                }
            );
        }
    }
    #[test]
    fn standard_failure_alone_allows_explicit_fallback_and_retains_failure() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let input = units("rgb(calc(1em) 0 0)");
        let ParseOutcome::StandardFailure(checkpoint) = parse(
            &mut owner,
            &input,
            InputKind::Utf8,
            GrammarRequest::WholeColor,
            &facts,
        )
        .unwrap() else {
            panic!("invalid channel type")
        };
        let failure = checkpoint.failure();
        assert_eq!(failure.reason, StandardReason::ComponentGrammar);
        assert_eq!(failure.site, Site::Source(Span { start: 4, end: 9 }));
        let checkpoint = checkpoint.fallback(Color::opaque_black());
        assert_eq!(checkpoint.standard_failure(), Some(failure));
        assert_eq!(
            serialize(checkpoint, 7),
            Ok(("#000000".to_owned(), 7, Some(failure)))
        );
        let mut other = CssColorOperationOwner::new();
        assert!(matches!(
            parse(
                &mut other,
                &units("currentcolor"),
                InputKind::Utf8,
                GrammarRequest::WholeColor,
                &facts
            ),
            Err(Error::Resolution(super::super::resolution::Error::Palette(
                super::super::palette::PaletteError::MissingCurrent
            )))
        ));
    }
    #[test]
    fn prefix_boundary_utf16_preprocessing_and_numerical_refusal_keep_actual_context() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let input = units(" /*x*/ red blue");
        let ParseOutcome::Used(checkpoint) = parse(
            &mut owner,
            &input,
            InputKind::Utf8,
            GrammarRequest::PrefixColor,
            &facts,
        )
        .unwrap() else {
            panic!("prefix")
        };
        assert_eq!(checkpoint.context().consumed, Span { start: 0, end: 10 });
        assert_eq!(checkpoint.context().input_length, input.len());
        assert!(std::ptr::eq(checkpoint.context().facts, &facts));
        assert_eq!(checkpoint.value().srgb_bytes(), Some([255, 0, 0]));
        let mut utf16 = CssColorOperationOwner::new();
        let mut input: Vec<_> = "rgb(1 /*".encode_utf16().collect();
        input.push(0xd800);
        input.extend("*/ 0 0)".encode_utf16());
        let ParseOutcome::Used(checkpoint) = parse(
            &mut utf16,
            &input,
            InputKind::Utf16,
            GrammarRequest::WholeColor,
            &facts,
        )
        .unwrap() else {
            panic!("comment preprocessing")
        };
        assert_eq!(checkpoint.context().input_kind, InputKind::Utf16);
        assert_eq!(checkpoint.context().consumed.end, input.len());
        assert_eq!(
            checkpoint.value().channels(),
            [Some(1.0 / 255.0), Some(0.0), Some(0.0)]
        );
        let mut numerical = CssColorOperationOwner::new();
        let ParseOutcome::Used(checkpoint) = parse(
            &mut numerical,
            &units("color(srgb-linear 1.7e308 1.7e308 1.7e308)"),
            InputKind::Utf8,
            GrammarRequest::WholeColor,
            &facts,
        )
        .unwrap() else {
            panic!("finite channels")
        };
        let original = checkpoint.value();
        // XYZ Z is 1.089 times the shared channel, beyond binary64, so it is
        // the closest finite value; X and Y (0.95 and 1.0 times) stay exact.
        let checkpoint = transform(
            checkpoint,
            Transformation::Convert(surgeist_color::ColorSpace::Predefined(
                surgeist_color::PredefinedColorSpace::XyzD65,
            )),
        );
        let [x, y, z] = checkpoint.value().channels();
        assert!(x.is_some_and(|x| x.is_finite() && x < f64::MAX));
        assert!(y.is_some_and(|y| y.is_finite() && y < f64::MAX));
        assert_eq!(z, Some(f64::MAX));
        assert!(!checkpoint.value().identical(original));
        assert_eq!(checkpoint.standard_failure(), None);
    }
    #[test]
    fn actual_foreign_or_wrong_bank_storage_refusal_retains_pending_action() {
        let facts = facts();
        let mut owner = CssColorOperationOwner::new();
        let mut other = CssColorOperationOwner::new();
        let mut cursor = ColorOperation::start(
            &mut owner,
            InputKind::Utf8,
            1,
            GrammarRequest::WholeColor,
            &facts,
            facts.identity,
        )
        .unwrap();
        let mut foreign = ColorOperation::start(
            &mut other,
            InputKind::Utf8,
            1,
            GrammarRequest::WholeColor,
            &facts,
            facts.identity,
        )
        .unwrap();
        for cursor in [&mut cursor, &mut foreign] {
            cursor.read_input(0, u16::from(b'r')).unwrap();
        }
        let expected = cursor.action();
        assert!(matches!(
            expected,
            Action::Lexical(LexicalAction::AppendSlot { index: 0, .. })
        ));
        let slot = foreign.pending_record(CssColorBank::Algorithm, 0).unwrap();
        assert_eq!(
            cursor.written(CssColorBank::Algorithm, 0, &slot),
            Err(Error::Protocol(ColorProtocolError::WrongSlot))
        );
        assert_eq!(cursor.action(), expected);
        assert_eq!(
            cursor.pending_record(CssColorBank::Arithmetic, 0).err(),
            Some(Error::Protocol(ColorProtocolError::WrongAction))
        );
        assert_eq!(cursor.action(), expected);
        let slot = cursor.pending_record(CssColorBank::Algorithm, 0).unwrap();
        cursor.written(CssColorBank::Algorithm, 0, &slot).unwrap();
        assert_eq!(cursor.bank_length(CssColorBank::Algorithm), 1);
        cursor.abandon();
        foreign.abandon();
        assert_eq!(cursor.action(), Action::Stopped);
        assert_eq!(
            cursor.compute(),
            Err(Error::Protocol(ColorProtocolError::WrongAction))
        );
        assert!(cursor.take().is_err());
    }
}
