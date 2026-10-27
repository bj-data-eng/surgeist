//! Iterative color recipes over the canonical decoded token owner.
//!
//! Recipes are algorithm records. Numeric component grammar/type validation and
//! reached resolution must finish before any record can become a used value.

use super::keywords::{ColorFunction as F, Word};
use super::tokens::{ClassifiedToken, TokenAction, TokenReader};
use crate::color::controlled::CssColorSlot;
use crate::color::controlled::protocol::{
    ColorProtocolError, Data, LexicalRecord, LexicalTable, RecordRange, RecordTable,
};
use crate::color::{CssColorInterpolationMethod, CssColorSyntax, CssSystemColor};
use crate::tokenization::tokenizer::{NameKind, NumberKind, Span, TokenKind};
use surgeist_color::ColorSpace as Space;
use surgeist_color::{HueInterpolation, PredefinedColorSpace};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RawComponent {
    Authored {
        site: usize,
    },
    Atom(ClassifiedToken),
    Math {
        opener: ClassifiedToken,
        end: usize,
        closed: bool,
    },
}
impl RawComponent {
    pub(crate) fn source_token(self) -> Result<ClassifiedToken, ColorProtocolError> {
        match self {
            Self::Atom(token) | Self::Math { opener: token, .. } => Ok(token),
            Self::Authored { .. } => Err(ColorProtocolError::WrongSlot),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ComponentRole {
    Channel,
    Hue,
    Alpha,
    Weight,
    RelativeChannel,
    ProfileChannel,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ComponentRecord {
    pub(crate) previous: Option<usize>,
    pub(crate) ordinal: usize,
    pub(crate) role: ComponentRole,
    pub(crate) component: RawComponent,
    pub(crate) context: super::component::ComponentContext,
    pub(crate) checked: Option<super::component::CheckedScalar>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Leaf {
    Current,
    Transparent,
    Named([u8; 3]),
    Hex([u8; 4]),
    System(CssSystemColor),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ProfileSpace {
    Predefined(PredefinedColorSpace),
    Custom { token: usize, units: RecordRange },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Interpolation {
    Standard(CssColorInterpolationMethod),
    Custom { token: usize, units: RecordRange },
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum MixSpace {
    Standard(Space),
    Custom { token: usize, units: Span },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FunctionRecipe {
    pub(crate) function: F,
    pub(crate) origin: Option<usize>,
    pub(crate) space: Option<ProfileSpace>,
    pub(crate) syntax: CssColorSyntax,
    pub(crate) components: Option<usize>,
    pub(crate) component_start: Option<usize>,
    pub(crate) count: usize,
    pub(crate) alpha: Option<usize>,
    pub(crate) children: [Option<usize>; 2],
    pub(crate) mix: Option<usize>,
    pub(crate) mix_count: usize,
    pub(crate) interpolation: Option<Interpolation>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum RecipeKind {
    Leaf(Leaf),
    Function(FunctionRecipe),
}
impl FunctionRecipe {
    pub(crate) fn component_index(self, ordinal: usize) -> Result<usize, ColorProtocolError> {
        if ordinal >= self.count {
            return Err(ColorProtocolError::WrongIndex);
        }
        self.component_start
            .ok_or(ColorProtocolError::WrongSlot)?
            .checked_add(ordinal)
            .ok_or(ColorProtocolError::Overflow)
    }
}
/// Provenance belongs to the actual input domain, independent of bank indices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Site {
    Source(Span),
    Authored { record: usize },
}
impl Site {
    pub(crate) fn source(self) -> Result<Span, ColorProtocolError> {
        match self {
            Self::Source(source) => Ok(source),
            Self::Authored { .. } => Err(ColorProtocolError::WrongSlot),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Recipe {
    pub(crate) kind: RecipeKind,
    pub(crate) site: Site,
    pub(crate) closed: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MixRecord {
    pub(crate) previous: Option<usize>,
    pub(crate) ordinal: usize,
    pub(crate) color: usize,
    pub(crate) weight: Option<usize>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FrameStage {
    Header,
    Origin,
    Space,
    Component,
    AfterComponent,
    Alpha,
    Close,
    Wrapper,
    Second,
    AfterWrapper,
    AlphaTail,
    MixHeader,
    MixSpace,
    MixHue,
    MixHueSuffix,
    MixComma,
    MixLeading,
    MixColor,
    MixTail,
    MixAfter,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Frame {
    parent: Option<usize>,
    recipe: FunctionRecipe,
    stage: FrameStage,
    source: Span,
    closed: bool,
    depth: usize,
    relative: bool,
    /// Whether the first component is `none`, which no legacy syntax admits.
    leading_none: bool,
    weight: Option<usize>,
    mix_color: Option<usize>,
    mix_space: Option<MixSpace>,
    mix_hue: Option<HueInterpolation>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum GrammarRecord {
    Frame(Frame),
    Recipe(Recipe),
    Component(ComponentRecord),
    Mix(MixRecord),
}

#[derive(Clone, Copy)]
pub(crate) struct GrammarTable<'owner> {
    pub(crate) input: GrammarInput<'owner>,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) root: usize,
}
#[derive(Clone, Copy)]
pub(crate) enum GrammarInput<'css> {
    Source(LexicalTable<'css>),
    Authored(RecordTable<'css>),
}
impl<'css> GrammarInput<'css> {
    pub(crate) const fn records(self) -> RecordTable<'css> {
        match self {
            Self::Source(table) => table.records(),
            Self::Authored(table) => table,
        }
    }
    pub(crate) fn source(self) -> Result<LexicalTable<'css>, ColorProtocolError> {
        match self {
            Self::Source(table) => Ok(table),
            Self::Authored(_) => Err(ColorProtocolError::WrongSlot),
        }
    }
}
impl GrammarTable<'_> {
    pub(crate) fn record(
        &self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<GrammarRecord, ColorProtocolError> {
        slot.qualify(
            self.input.records().owner,
            self.input.records().phase,
            index,
            crate::color::controlled::CssColorBank::Algorithm,
        )?;
        if index < self.start || index >= self.end {
            return Err(ColorProtocolError::WrongIndex);
        }
        let Data::Grammar(record) = slot.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        Ok(record)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GrammarFailureReason {
    Syntax,
    HueInterpolation,
    ColorSpace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GrammarAction {
    ReadLexical { index: usize },
    ReadFrame { index: usize },
    ReadHex { index: usize },
    Append { index: usize },
    Compute,
    Done { recipe: usize },
    Failed { source: Span },
    Stopped,
}

#[derive(Clone, Copy)]
enum Commit {
    Suspend,
    Color { parent: Option<usize>, closed: bool },
    Component { frame: Frame, role: ComponentRole },
    Mix { frame: Frame },
}
#[derive(Clone, Copy)]
enum State {
    Color {
        parent: Option<usize>,
        depth: usize,
    },
    Frame(Frame),
    Math {
        frame: Frame,
        role: ComponentRole,
        opener: ClassifiedToken,
        depth: usize,
        end: usize,
    },
    Hex {
        parent: Option<usize>,
        token: ClassifiedToken,
        units: Span,
        index: usize,
        bytes: [u8; 4],
        length: usize,
    },
    Pending {
        record: GrammarRecord,
        commit: Commit,
    },
    Parent {
        index: usize,
        child: usize,
        closed: bool,
    },
    End {
        recipe: usize,
    },
    Done {
        recipe: usize,
    },
    Failed {
        source: Span,
        reason: GrammarFailureReason,
    },
    Stopped,
}

pub(crate) struct ColorGrammar<'owner> {
    table: LexicalTable<'owner>,
    reader: TokenReader<'owner>,
    state: State,
    next_record: usize,
    last_end: usize,
    whole: bool,
}
impl<'owner> ColorGrammar<'owner> {
    pub(crate) fn start(table: &LexicalTable<'owner>, whole: bool) -> Self {
        Self {
            table: *table,
            reader: TokenReader::start(table),
            state: State::Color {
                parent: None,
                depth: 0,
            },
            next_record: table.length(),
            last_end: 0,
            whole,
        }
    }
    pub(crate) fn action(&self) -> GrammarAction {
        match self.state {
            State::Pending { .. } => GrammarAction::Append {
                index: self.next_record,
            },
            State::Parent { index, .. } => GrammarAction::ReadFrame { index },
            State::Hex { index, units, .. } if index < units.end => {
                GrammarAction::ReadHex { index }
            }
            State::Hex { .. } => GrammarAction::Compute,
            State::Done { recipe } => GrammarAction::Done { recipe },
            State::Failed { source, .. } => GrammarAction::Failed { source },
            State::Stopped => GrammarAction::Stopped,
            _ => match self.reader.action() {
                TokenAction::ReadRecord { index } | TokenAction::ReadName { index } => {
                    GrammarAction::ReadLexical { index }
                }
                _ => GrammarAction::Compute,
            },
        }
    }
    pub(crate) fn read_lexical(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if self.action() != (GrammarAction::ReadLexical { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        self.reader.read(index, slot)
    }
    pub(crate) fn record(&self, index: usize) -> Result<CssColorSlot<'owner>, ColorProtocolError> {
        if self.action() != (GrammarAction::Append { index }) {
            return Err(ColorProtocolError::WrongIndex);
        }
        self.next_record
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        let State::Pending { record, commit } = self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        // Every counter required by the acknowledged transition is checked
        // before the caller can commit storage for this actual record.
        match commit {
            Commit::Suspend => {
                let GrammarRecord::Frame(frame) = record else {
                    return Err(ColorProtocolError::WrongSlot);
                };
                frame
                    .depth
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
            }
            Commit::Component { frame, role }
                if !matches!(role, ComponentRole::Alpha | ComponentRole::Weight) =>
            {
                frame
                    .recipe
                    .count
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                if let Some(start) = frame.recipe.component_start {
                    if start.checked_add(frame.recipe.count) != Some(index)
                        || frame.recipe.components != index.checked_sub(1)
                    {
                        return Err(ColorProtocolError::WrongIndex);
                    }
                } else if frame.recipe.count != 0 || frame.recipe.components.is_some() {
                    return Err(ColorProtocolError::WrongSlot);
                }
            }
            Commit::Mix { frame } => {
                frame
                    .recipe
                    .mix_count
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
            }
            _ => {}
        }
        Ok(CssColorSlot::new(
            self.table.owner,
            self.table.phase,
            index,
            Data::Grammar(record),
        ))
    }
    pub(crate) fn appended(
        &mut self,
        index: usize,
        record: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        self.qualify(index, record)?;
        if index != self.next_record {
            return Err(ColorProtocolError::WrongIndex);
        }
        let State::Pending {
            record: expected,
            commit,
        } = self.state
        else {
            return Err(ColorProtocolError::WrongAction);
        };
        let Data::Grammar(actual) = record.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        if expected != actual {
            return Err(ColorProtocolError::WrongSlot);
        }
        let next = self
            .next_record
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        self.state = match commit {
            Commit::Suspend => {
                let GrammarRecord::Frame(frame) = expected else {
                    return Err(ColorProtocolError::WrongSlot);
                };
                State::Color {
                    parent: Some(index),
                    depth: frame
                        .depth
                        .checked_add(1)
                        .ok_or(ColorProtocolError::Overflow)?,
                }
            }
            Commit::Color {
                parent: Some(parent),
                closed,
            } => State::Parent {
                index: parent,
                child: index,
                closed,
            },
            Commit::Color { parent: None, .. } => {
                if self.whole {
                    State::End { recipe: index }
                } else {
                    State::Done { recipe: index }
                }
            }
            Commit::Component { mut frame, role } => {
                let GrammarRecord::Component(_) = expected else {
                    return Err(ColorProtocolError::WrongSlot);
                };
                if role == ComponentRole::Alpha {
                    frame.recipe.alpha = Some(index);
                    frame.stage = FrameStage::Close;
                } else if role == ComponentRole::Weight {
                    frame.weight = Some(index);
                    frame.stage = if frame.mix_color.is_some() {
                        FrameStage::MixAfter
                    } else {
                        FrameStage::MixColor
                    };
                } else {
                    frame.recipe.component_start.get_or_insert(index);
                    frame.recipe.components = Some(index);
                    frame.recipe.count = frame
                        .recipe
                        .count
                        .checked_add(1)
                        .ok_or(ColorProtocolError::Overflow)?;
                    frame.stage = FrameStage::AfterComponent;
                }
                State::Frame(frame)
            }
            Commit::Mix { mut frame } => {
                frame.recipe.mix = Some(index);
                frame.recipe.mix_count = frame
                    .recipe
                    .mix_count
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                frame.weight = None;
                frame.mix_color = None;
                frame.stage = FrameStage::MixAfter;
                State::Frame(frame)
            }
        };
        self.next_record = next;
        Ok(())
    }
    fn qualify(&self, index: usize, slot: &CssColorSlot<'_>) -> Result<(), ColorProtocolError> {
        slot.qualify(self.table.owner, self.table.phase, index, slot.data.bank())?;
        if index != slot.index {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(())
    }
    pub(crate) fn read_frame(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        self.qualify(index, slot)?;
        let State::Parent {
            index: expected,
            child,
            closed,
        } = self.state
        else {
            return Err(ColorProtocolError::WrongAction);
        };
        if index != expected {
            return Err(ColorProtocolError::WrongIndex);
        }
        let Data::Grammar(GrammarRecord::Frame(mut frame)) = slot.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        frame.closed &= closed;
        frame.stage = match frame.stage {
            FrameStage::Origin => {
                frame.recipe.origin = Some(child);
                if frame.recipe.function == F::Alpha {
                    FrameStage::AlphaTail
                } else if frame.recipe.function == F::Color {
                    FrameStage::Space
                } else {
                    FrameStage::Component
                }
            }
            FrameStage::Wrapper => {
                frame.recipe.children[0] = Some(child);
                if frame.recipe.function == F::LightDark {
                    FrameStage::Second
                } else {
                    FrameStage::Close
                }
            }
            FrameStage::AfterWrapper => {
                frame.recipe.children[1] = Some(child);
                FrameStage::Close
            }
            FrameStage::MixColor => {
                frame.mix_color = Some(child);
                FrameStage::MixTail
            }
            _ => return Err(ColorProtocolError::WrongAction),
        };
        self.state = State::Frame(frame);
        Ok(())
    }
    fn token(&self) -> Result<Option<ClassifiedToken>, ColorProtocolError> {
        match self.reader.action() {
            TokenAction::Yield => self.reader.current().map(Some),
            TokenAction::End => Ok(None),
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    fn consume(&mut self) -> Result<(), ColorProtocolError> {
        self.last_end = self.reader.current()?.token.source.end;
        self.reader.consumed()
    }
    pub(crate) fn failure_reason(
        &self,
        source: Span,
    ) -> Result<GrammarFailureReason, ColorProtocolError> {
        let State::Failed {
            source: actual,
            reason,
        } = self.state
        else {
            return Err(ColorProtocolError::WrongAction);
        };
        if source != actual {
            return Err(ColorProtocolError::WrongAction);
        }
        Ok(reason)
    }
    fn failure(&mut self, token: Option<ClassifiedToken>) {
        self.failure_with_reason(token, GrammarFailureReason::Syntax);
    }
    fn failure_with_reason(
        &mut self,
        token: Option<ClassifiedToken>,
        reason: GrammarFailureReason,
    ) {
        self.state = State::Failed {
            source: token.map_or(
                Span {
                    start: self.last_end,
                    end: self.last_end,
                },
                |value| value.token.source,
            ),
            reason,
        };
    }
    fn suspend(&mut self, frame: Frame) {
        self.state = State::Pending {
            record: GrammarRecord::Frame(frame),
            commit: Commit::Suspend,
        };
    }
    fn component(
        &mut self,
        frame: Frame,
        role: ComponentRole,
        component: RawComponent,
    ) -> Result<(), ColorProtocolError> {
        let context =
            super::component::ComponentContext::for_recipe(frame.recipe, role, frame.recipe.count)?;
        let previous = if matches!(role, ComponentRole::Alpha | ComponentRole::Weight) {
            None
        } else {
            frame.recipe.components
        };
        self.state = State::Pending {
            record: GrammarRecord::Component(ComponentRecord {
                previous,
                ordinal: frame.recipe.count,
                role,
                component,
                context,
                checked: None,
            }),
            commit: Commit::Component { frame, role },
        };
        Ok(())
    }
    fn numeric(
        &mut self,
        frame: Frame,
        role: ComponentRole,
        token: Option<ClassifiedToken>,
    ) -> Result<(), ColorProtocolError> {
        let Some(token) = token else {
            self.failure(None);
            return Ok(());
        };
        let allowed_ident = token.word == Word::None
            || frame.relative
            || matches!(frame.recipe.space, Some(ProfileSpace::Custom { .. })) && frame.relative;
        match token.token.kind {
            TokenKind::Number { .. }
            | TokenKind::Name {
                kind: NameKind::Ident,
                ..
            } if matches!(token.token.kind, TokenKind::Number { .. }) || allowed_ident => {
                self.consume()?;
                self.component(frame, role, RawComponent::Atom(token))?;
            }
            TokenKind::Name {
                kind: NameKind::Function,
                ..
            } if matches!(token.word, Word::Math(_)) => {
                self.consume()?;
                self.state = State::Math {
                    frame,
                    role,
                    opener: token,
                    depth: 1,
                    end: token.index,
                };
            }
            _ => self.failure(Some(token)),
        }
        Ok(())
    }
    fn close(
        &mut self,
        mut frame: Frame,
        token: Option<ClassifiedToken>,
    ) -> Result<(), ColorProtocolError> {
        if token.is_some_and(|value| value.token.kind != TokenKind::Delimiter(')')) {
            self.failure(token);
            return Ok(());
        }
        if let Some(token) = token {
            frame.source.end = token.token.source.end;
            self.consume()?;
        } else {
            frame.source.end = self.last_end;
            frame.closed = false;
        }
        self.state = State::Pending {
            record: GrammarRecord::Recipe(Recipe {
                kind: RecipeKind::Function(frame.recipe),
                site: Site::Source(frame.source),
                closed: frame.closed,
            }),
            commit: Commit::Color {
                parent: frame.parent,
                closed: frame.closed,
            },
        };
        Ok(())
    }
    pub(crate) fn compute(&mut self) -> Result<(), ColorProtocolError> {
        if self.action() != GrammarAction::Compute {
            return Err(ColorProtocolError::WrongAction);
        }
        if let State::Hex {
            parent,
            token,
            units,
            index,
            bytes,
            length,
        } = self.state
        {
            if index != units.end {
                return Err(ColorProtocolError::WrongAction);
            }
            let value = super::hex::finish(bytes, length);
            self.state = State::Pending {
                record: GrammarRecord::Recipe(Recipe {
                    kind: RecipeKind::Leaf(Leaf::Hex(value)),
                    site: Site::Source(token.token.source),
                    closed: true,
                }),
                commit: Commit::Color {
                    parent,
                    closed: true,
                },
            };
            return Ok(());
        }
        if self.reader.action() == TokenAction::Classify {
            return self.reader.classify();
        }
        let token = self.token()?;
        if token.is_some_and(|value| {
            matches!(
                value.token.kind,
                TokenKind::Whitespace | TokenKind::Comment { .. }
            )
        }) {
            if let State::Math {
                frame,
                role,
                opener,
                depth,
                ..
            } = self.state
            {
                self.state = State::Math {
                    frame,
                    role,
                    opener,
                    depth,
                    end: token.expect("present trivia").index,
                };
            }
            return self.consume();
        }
        match self.state {
            State::Color { parent, depth } => {
                let Some(token) = token else {
                    self.failure(None);
                    return Ok(());
                };
                if let TokenKind::Name {
                    kind: NameKind::Function,
                    ..
                } = token.token.kind
                    && let Word::Function(function) = token.word
                {
                    self.consume()?;
                    self.state = State::Frame(Frame {
                        parent,
                        recipe: FunctionRecipe {
                            function,
                            origin: None,
                            space: None,
                            syntax: CssColorSyntax::Modern,
                            components: None,
                            component_start: None,
                            count: 0,
                            alpha: None,
                            children: [None; 2],
                            mix: None,
                            mix_count: 0,
                            interpolation: None,
                        },
                        stage: FrameStage::Header,
                        source: token.token.source,
                        closed: true,
                        depth,
                        relative: false,
                        leading_none: false,
                        weight: None,
                        mix_color: None,
                        mix_space: None,
                        mix_hue: None,
                    });
                } else if let TokenKind::Name {
                    kind: NameKind::Hash { .. },
                    units,
                } = token.token.kind
                {
                    let length = units
                        .end
                        .checked_sub(units.start)
                        .ok_or(ColorProtocolError::Overflow)?;
                    if !matches!(length, 3 | 4 | 6 | 8) {
                        self.failure(Some(token));
                        return Ok(());
                    }
                    self.consume()?;
                    self.state = State::Hex {
                        parent,
                        token,
                        units,
                        index: units.start,
                        bytes: [0; 4],
                        length,
                    };
                } else if matches!(
                    token.token.kind,
                    TokenKind::Name {
                        kind: NameKind::Ident,
                        ..
                    }
                ) {
                    let leaf = match token.word {
                        Word::Current => Leaf::Current,
                        Word::Transparent => Leaf::Transparent,
                        Word::Named(value) => Leaf::Named(value),
                        Word::System(value) => Leaf::System(value),
                        _ => {
                            self.failure(Some(token));
                            return Ok(());
                        }
                    };
                    self.consume()?;
                    self.state = State::Pending {
                        record: GrammarRecord::Recipe(Recipe {
                            kind: RecipeKind::Leaf(leaf),
                            site: Site::Source(token.token.source),
                            closed: true,
                        }),
                        commit: Commit::Color {
                            parent,
                            closed: true,
                        },
                    };
                } else {
                    self.failure(Some(token));
                }
            }
            State::End { recipe } => {
                if token.is_none() {
                    self.state = State::Done { recipe };
                } else {
                    self.failure(token);
                }
            }
            State::Math {
                frame,
                role,
                opener,
                depth,
                end,
            } => {
                if let Some(token) = token {
                    let nested = matches!(
                        token.token.kind,
                        TokenKind::Name {
                            kind: NameKind::Function,
                            ..
                        } | TokenKind::Delimiter('(')
                    );
                    let next_depth = if nested {
                        depth.checked_add(1).ok_or(ColorProtocolError::Overflow)?
                    } else if token.token.kind == TokenKind::Delimiter(')') {
                        depth - 1
                    } else {
                        depth
                    };
                    self.consume()?;
                    if next_depth == 0 {
                        self.component(
                            frame,
                            role,
                            RawComponent::Math {
                                opener,
                                end: token.index,
                                closed: true,
                            },
                        )?;
                    } else {
                        self.state = State::Math {
                            frame,
                            role,
                            opener,
                            depth: next_depth,
                            end: token.index,
                        };
                    }
                } else {
                    let mut frame = frame;
                    frame.closed = false;
                    self.component(
                        frame,
                        role,
                        RawComponent::Math {
                            opener,
                            end,
                            closed: false,
                        },
                    )?;
                }
            }
            State::Frame(frame) => self.frame(frame, token)?,
            _ => return Err(ColorProtocolError::WrongAction),
        }
        Ok(())
    }
    pub(crate) fn read_hex(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if self.action() != (GrammarAction::ReadHex { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        let LexicalRecord::NameUnit(unit) = self.table.observe(index, slot)? else {
            return Err(ColorProtocolError::WrongSlot);
        };
        let State::Hex {
            parent,
            token,
            units,
            mut bytes,
            length,
            ..
        } = self.state
        else {
            return Err(ColorProtocolError::WrongAction);
        };
        let position = index - units.start;
        if !super::hex::write_digit(&mut bytes, length, position, unit) {
            self.failure(Some(token));
            return Ok(());
        }
        self.state = State::Hex {
            parent,
            token,
            units,
            index: index + 1,
            bytes,
            length,
        };
        Ok(())
    }
    fn frame(
        &mut self,
        mut frame: Frame,
        token: Option<ClassifiedToken>,
    ) -> Result<(), ColorProtocolError> {
        let word = token.map(|value| value.word);
        let delimiter = token.and_then(|value| {
            if let TokenKind::Delimiter(unit) = value.token.kind {
                Some(unit)
            } else {
                None
            }
        });
        let closing = token.is_none() || delimiter == Some(')');
        match frame.stage {
            FrameStage::Header => {
                if matches!(frame.recipe.function, F::Contrast | F::LightDark) {
                    frame.stage = FrameStage::Wrapper;
                    self.suspend(frame);
                } else if frame.recipe.function == F::Mix {
                    frame.stage = FrameStage::MixHeader;
                    self.state = State::Frame(frame);
                } else if word == Some(Word::From)
                    && !matches!(frame.recipe.function, F::DeviceCmyk)
                {
                    self.consume()?;
                    frame.relative = true;
                    frame.stage = FrameStage::Origin;
                    self.suspend(frame);
                } else if frame.recipe.function == F::Alpha {
                    self.failure(token);
                } else {
                    frame.stage = if frame.recipe.function == F::Color {
                        FrameStage::Space
                    } else {
                        FrameStage::Component
                    };
                    self.state = State::Frame(frame);
                }
            }
            FrameStage::Space => {
                let Some(value) = token else {
                    self.failure_with_reason(None, GrammarFailureReason::ColorSpace);
                    return Ok(());
                };
                let space = match (value.token.kind, value.word) {
                    (
                        TokenKind::Name {
                            kind: NameKind::Ident,
                            ..
                        },
                        Word::Predefined(space),
                    ) => ProfileSpace::Predefined(space),
                    (
                        TokenKind::Name {
                            kind: NameKind::Ident,
                            units,
                        },
                        _,
                    ) if value.dashed => ProfileSpace::Custom {
                        token: value.index,
                        units: RecordRange {
                            start: units.start,
                            end: units.end,
                        },
                    },
                    _ => {
                        self.failure_with_reason(Some(value), GrammarFailureReason::ColorSpace);
                        return Ok(());
                    }
                };
                self.consume()?;
                frame.recipe.space = Some(space);
                frame.stage = FrameStage::Component;
                self.state = State::Frame(frame);
            }
            FrameStage::Component => {
                if closing {
                    if matches!(frame.recipe.space, Some(ProfileSpace::Custom { .. }))
                        && frame.recipe.count != 0
                    {
                        self.close(frame, token)?;
                    } else {
                        self.failure(token);
                    }
                } else {
                    let role = if frame.relative {
                        ComponentRole::RelativeChannel
                    } else if matches!(frame.recipe.space, Some(ProfileSpace::Custom { .. })) {
                        ComponentRole::ProfileChannel
                    } else if frame.recipe.count == 0
                        && matches!(frame.recipe.function, F::Hsl | F::Hwb)
                        || frame.recipe.count == 2
                            && matches!(frame.recipe.function, F::Lch | F::Oklch)
                    {
                        ComponentRole::Hue
                    } else {
                        ComponentRole::Channel
                    };
                    if frame.recipe.count == 0 {
                        frame.leading_none = token.is_some_and(|value| value.word == Word::None);
                    }
                    self.numeric(frame, role, token)?;
                }
            }
            FrameStage::AfterComponent => {
                let custom = matches!(frame.recipe.space, Some(ProfileSpace::Custom { .. }));
                let complete = if custom {
                    closing || delimiter == Some('/')
                } else {
                    frame.recipe.count
                        == if frame.recipe.function == F::DeviceCmyk {
                            4
                        } else {
                            3
                        }
                };
                if complete {
                    if delimiter == Some('/') && frame.recipe.syntax == CssColorSyntax::Modern
                        || delimiter == Some(',')
                            && frame.recipe.syntax == CssColorSyntax::Legacy
                            && frame.recipe.function != F::DeviceCmyk
                    {
                        self.consume()?;
                        frame.stage = FrameStage::Alpha;
                        self.state = State::Frame(frame);
                    } else {
                        frame.stage = FrameStage::Close;
                        self.state = State::Frame(frame);
                    }
                } else if delimiter == Some(',')
                    && !frame.relative
                    && matches!(frame.recipe.function, F::Rgb | F::Hsl | F::DeviceCmyk)
                {
                    if frame.recipe.count == 1 {
                        // Color 4 §5.1, §7: legacy syntax admits no `none`, so
                        // after a leading `none` the modern alternative reaches
                        // furthest and fails at this comma.
                        if frame.leading_none {
                            self.failure(token);
                            return Ok(());
                        }
                        frame.recipe.syntax = CssColorSyntax::Legacy;
                    }
                    if frame.recipe.syntax != CssColorSyntax::Legacy {
                        self.failure(token);
                    } else {
                        self.consume()?;
                        frame.stage = FrameStage::Component;
                        self.state = State::Frame(frame);
                    }
                } else if frame.recipe.syntax == CssColorSyntax::Legacy {
                    self.failure(token);
                } else {
                    frame.stage = FrameStage::Component;
                    self.state = State::Frame(frame);
                }
            }
            FrameStage::Alpha => self.numeric(frame, ComponentRole::Alpha, token)?,
            FrameStage::Close => self.close(frame, token)?,
            FrameStage::Second => {
                if delimiter == Some(',') {
                    self.consume()?;
                    frame.stage = FrameStage::AfterWrapper;
                    self.suspend(frame);
                } else {
                    self.failure(token);
                }
            }
            FrameStage::AlphaTail => {
                if delimiter == Some('/') {
                    self.consume()?;
                    frame.stage = FrameStage::Alpha;
                    self.state = State::Frame(frame);
                } else {
                    frame.stage = FrameStage::Close;
                    self.state = State::Frame(frame);
                }
            }
            FrameStage::MixHeader => {
                if word == Some(Word::In) {
                    self.consume()?;
                    frame.stage = FrameStage::MixSpace;
                    self.state = State::Frame(frame);
                } else {
                    frame.stage = FrameStage::MixLeading;
                    self.state = State::Frame(frame);
                }
            }
            FrameStage::MixSpace => {
                if let Some(value) = token.filter(|value| value.dashed) {
                    let TokenKind::Name {
                        kind: NameKind::Ident,
                        units,
                    } = value.token.kind
                    else {
                        self.failure(token);
                        return Ok(());
                    };
                    self.consume()?;
                    frame.mix_space = Some(MixSpace::Custom {
                        token: value.index,
                        units,
                    });
                    frame.stage = FrameStage::MixHue;
                    self.state = State::Frame(frame);
                    return Ok(());
                }
                let space = match word {
                    Some(Word::Predefined(space)) => Space::Predefined(space),
                    Some(Word::Interpolation(space)) => space,
                    _ => {
                        self.failure(token);
                        return Ok(());
                    }
                };
                if !token.is_some_and(|value| {
                    matches!(
                        value.token.kind,
                        TokenKind::Name {
                            kind: NameKind::Ident,
                            ..
                        }
                    )
                }) {
                    self.failure(token);
                    return Ok(());
                }
                self.consume()?;
                frame.mix_space = Some(MixSpace::Standard(space));
                frame.stage = FrameStage::MixHue;
                self.state = State::Frame(frame);
            }
            FrameStage::MixHue => {
                if let Some(Word::Hue(hue)) = word {
                    let permitted = match frame.mix_space.ok_or(ColorProtocolError::WrongAction)? {
                        MixSpace::Standard(space) => {
                            CssColorInterpolationMethod::try_new(space, Some(hue)).is_ok()
                        }
                        MixSpace::Custom { .. } => false,
                    };
                    if !permitted {
                        self.failure_with_reason(token, GrammarFailureReason::HueInterpolation);
                        return Ok(());
                    }
                    self.consume()?;
                    frame.mix_hue = Some(hue);
                    frame.stage = FrameStage::MixHueSuffix;
                    self.state = State::Frame(frame);
                } else {
                    frame.stage = FrameStage::MixComma;
                    self.state = State::Frame(frame);
                }
            }
            FrameStage::MixHueSuffix => {
                if word == Some(Word::HueSuffix) {
                    self.consume()?;
                    frame.stage = FrameStage::MixComma;
                    self.state = State::Frame(frame);
                } else {
                    self.failure(token);
                }
            }
            FrameStage::MixComma => {
                if delimiter == Some(',') {
                    let method = match frame.mix_space.ok_or(ColorProtocolError::WrongAction)? {
                        MixSpace::Standard(space) => {
                            let Ok(method) =
                                CssColorInterpolationMethod::try_new(space, frame.mix_hue)
                            else {
                                self.failure(token);
                                return Ok(());
                            };
                            Interpolation::Standard(method)
                        }
                        MixSpace::Custom {
                            token: profile,
                            units,
                        } if frame.mix_hue.is_none() => Interpolation::Custom {
                            token: profile,
                            units: RecordRange {
                                start: units.start,
                                end: units.end,
                            },
                        },
                        _ => {
                            self.failure(token);
                            return Ok(());
                        }
                    };
                    self.consume()?;
                    frame.recipe.interpolation = Some(method);
                    frame.stage = FrameStage::MixLeading;
                    self.state = State::Frame(frame);
                } else {
                    self.failure(token);
                }
            }
            FrameStage::MixLeading => {
                if token.is_some_and(is_weight) {
                    self.numeric(frame, ComponentRole::Weight, token)?;
                } else {
                    frame.stage = FrameStage::MixColor;
                    self.suspend(frame);
                }
            }
            FrameStage::MixTail => {
                if frame.weight.is_none() && token.is_some_and(is_weight) {
                    self.numeric(frame, ComponentRole::Weight, token)?;
                } else {
                    self.mix_record(frame)?;
                }
            }
            FrameStage::MixAfter if frame.mix_color.is_some() => self.mix_record(frame)?,
            FrameStage::MixAfter => {
                if delimiter == Some(',') {
                    self.consume()?;
                    frame.stage = FrameStage::MixLeading;
                    self.state = State::Frame(frame);
                } else if closing && frame.recipe.mix_count != 0 {
                    self.close(frame, token)?;
                } else {
                    self.failure(token);
                }
            }
            FrameStage::MixColor => self.suspend(frame),
            FrameStage::Origin | FrameStage::Wrapper | FrameStage::AfterWrapper => {
                return Err(ColorProtocolError::WrongAction);
            }
        }
        Ok(())
    }
    fn mix_record(&mut self, frame: Frame) -> Result<(), ColorProtocolError> {
        let color = frame.mix_color.ok_or(ColorProtocolError::WrongAction)?;
        self.state = State::Pending {
            record: GrammarRecord::Mix(MixRecord {
                previous: frame.recipe.mix,
                ordinal: frame.recipe.mix_count,
                color,
                weight: frame.weight,
            }),
            commit: Commit::Mix { frame },
        };
        Ok(())
    }
    pub(crate) fn abandon(&mut self) {
        self.reader.abandon();
        self.state = State::Stopped;
    }
    pub(crate) fn finish(&self) -> Result<GrammarTable<'owner>, ColorProtocolError> {
        let State::Done { recipe } = self.state else {
            return Err(ColorProtocolError::Terminal);
        };
        Ok(GrammarTable {
            input: GrammarInput::Source(self.table),
            start: self.table.length(),
            end: self.next_record,
            root: recipe,
        })
    }
}
fn is_weight(token: ClassifiedToken) -> bool {
    matches!(
        token.token.kind,
        TokenKind::Number {
            kind: NumberKind::Percentage,
            ..
        }
    ) || matches!(
        token.token.kind,
        TokenKind::Name {
            kind: NameKind::Function,
            ..
        }
    ) && matches!(token.word, Word::Math(_))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::controlled::CssColorOperationOwner;
    use crate::color::controlled::protocol::{ActionDescription, LexicalOperation};
    use crate::tokenization::bounded::InputKind;

    fn parse(
        input: &str,
        whole: bool,
    ) -> Result<(Recipe, Vec<GrammarRecord>), (Span, GrammarFailureReason)> {
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
        let table = lexical.finish().unwrap();
        let mut grammar = ColorGrammar::start(&table, whole);
        for _ in 0..100_000 {
            match grammar.action() {
                GrammarAction::ReadLexical { index } => {
                    grammar.read_lexical(index, &slots[index]).unwrap()
                }
                GrammarAction::ReadFrame { index } => {
                    grammar.read_frame(index, &slots[index]).unwrap()
                }
                GrammarAction::ReadHex { index } => grammar.read_hex(index, &slots[index]).unwrap(),
                GrammarAction::Append { index } => {
                    assert_eq!(index, slots.len());
                    let record = grammar.record(index).unwrap();
                    // A refused append neither consumes the recipe nor advances.
                    assert_eq!(grammar.action(), GrammarAction::Append { index });
                    slots.push(record);
                    grammar.appended(index, &slots[index]).unwrap();
                }
                GrammarAction::Compute => grammar.compute().unwrap(),
                GrammarAction::Done { recipe } => {
                    let Data::Grammar(GrammarRecord::Recipe(value)) = slots[recipe].data else {
                        panic!("recipe")
                    };
                    let records = slots
                        .iter()
                        .filter_map(|slot| {
                            if let Data::Grammar(record) = slot.data {
                                Some(record)
                            } else {
                                None
                            }
                        })
                        .collect();
                    grammar.abandon();
                    assert_eq!(grammar.action(), GrammarAction::Stopped);
                    return Ok((value, records));
                }
                GrammarAction::Failed { source } => {
                    let reason = grammar.failure_reason(source).unwrap();
                    let different = Span {
                        start: source.start + 1,
                        end: source.end + 1,
                    };
                    assert_eq!(
                        grammar.failure_reason(different),
                        Err(ColorProtocolError::WrongAction)
                    );
                    grammar.abandon();
                    assert_eq!(
                        grammar.failure_reason(source),
                        Err(ColorProtocolError::WrongAction)
                    );
                    return Err((source, reason));
                }
                GrammarAction::Stopped => panic!("grammar stop"),
            }
        }
        panic!("finite fixture did not terminate")
    }

    #[test]
    fn literal_leaf_values_and_prefix_whole_discrimination() {
        for (input, expected) in [
            ("red", Leaf::Named([255, 0, 0])),
            ("currentcolor", Leaf::Current),
            ("transparent", Leaf::Transparent),
            ("#12a", Leaf::Hex([17, 34, 170, 255])),
            ("#1234", Leaf::Hex([17, 34, 51, 68])),
            ("#0123ab80", Leaf::Hex([1, 35, 171, 128])),
            ("\\43 anvasText", Leaf::System(CssSystemColor::CanvasText)),
        ] {
            let (recipe, _) = parse(input, true).unwrap();
            assert_eq!(recipe.kind, RecipeKind::Leaf(expected));
            assert!(recipe.closed);
        }
        assert!(parse("red blue", true).is_err());
        assert!(parse("red blue", false).is_ok());
        for input in [
            "#12",
            "#12345",
            "#g12",
            "garbage",
            "",
            "rgb()",
            "rgb(1 2)",
            "rgb(1,2 3)",
        ] {
            assert!(parse(input, true).is_err(), "{input}");
        }
    }

    #[test]
    fn all_function_families_keep_actual_structure_and_implicit_closure() {
        for (input, function, count, relative) in [
            ("rgb(1 2 3 / none)", F::Rgb, 3, false),
            ("hsl(1,2%,3%)", F::Hsl, 3, false),
            ("hwb(1 2% 3%)", F::Hwb, 3, false),
            ("lab(1 2 3)", F::Lab, 3, false),
            ("lch(1 2 3)", F::Lch, 3, false),
            ("oklab(1 2 3)", F::Oklab, 3, false),
            ("oklch(1 2 3)", F::Oklch, 3, false),
            ("color(display-p3 1 0 0)", F::Color, 3, false),
            ("color(--exact 1 2 3 4)", F::Color, 4, false),
            ("device-cmyk(0 1 1 0)", F::DeviceCmyk, 4, false),
            ("rgb(from red r g b)", F::Rgb, 3, true),
            ("color(from blue srgb r g b)", F::Color, 3, true),
            ("alpha(from red / calc(alpha / 2))", F::Alpha, 0, true),
            ("contrast-color(red)", F::Contrast, 0, false),
            ("light-dark(red,blue)", F::LightDark, 0, false),
        ] {
            let (recipe, _) = parse(input, true).unwrap();
            let RecipeKind::Function(value) = recipe.kind else {
                panic!("{input}")
            };
            assert_eq!(value.function, function, "{input}");
            assert_eq!(value.count, count, "{input}");
            assert_eq!(value.origin.is_some(), relative, "{input}");
            assert!(recipe.closed, "{input}");
        }
        for input in [
            "rgb(1 2 3",
            "rgb(1 2 calc(1 + 2",
            "light-dark(red, rgb(1 2 3",
        ] {
            let (recipe, _) = parse(input, true).unwrap();
            assert!(!recipe.closed, "{input}");
        }
    }

    #[test]
    fn variadic_mix_and_nested_origin_keep_real_links_and_weights() {
        let (recipe, records) = parse(
            "color-mix(in oklch longer hue, 20% rgb(from red r g b), blue 30%, white)",
            true,
        )
        .unwrap();
        let RecipeKind::Function(value) = recipe.kind else {
            panic!("mix")
        };
        assert_eq!(value.function, F::Mix);
        assert_eq!(value.mix_count, 3);
        assert_eq!(
            value.interpolation,
            Some(Interpolation::Standard(
                CssColorInterpolationMethod::try_new(Space::Oklch, Some(HueInterpolation::Longer))
                    .unwrap()
            ))
        );
        let members: Vec<_> = records
            .into_iter()
            .filter_map(|record| {
                if let GrammarRecord::Mix(member) = record {
                    Some(member)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(members.len(), 3);
        assert_eq!(members[0].ordinal, 0);
        assert!(members[0].previous.is_none());
        assert!(members[0].weight.is_some());
        assert!(members[1].weight.is_some());
        assert!(members[2].weight.is_none());
    }
    #[test]
    fn custom_mix_space_retains_actual_identifier_before_reached_metadata_resolution() {
        let (recipe, _) = parse("color-mix(in --\\65 xact, red, blue)", true).unwrap();
        let RecipeKind::Function(function) = recipe.kind else {
            panic!("mix")
        };
        assert!(
            matches!(function.interpolation, Some(Interpolation::Custom { units, .. }) if units.end - units.start == 7)
        );
        assert_eq!(function.mix_count, 2);
        // Custom spaces are rectangular. Hue interpolation keywords cannot
        // silently be applied to an arbitrary profile coordinate list.
        assert!(parse("color-mix(in --p longer hue, red, blue)", true).is_err());
    }
    #[test]
    fn rectangular_hue_failure_keeps_responsible_strategy_and_terminal_reason() {
        for input in [
            "color-mix(in srgb shorter hue, red, blue)",
            "color-mix(in --profile longer hue, red, blue)",
        ] {
            let strategy = if input.contains("shorter") {
                "shorter"
            } else {
                "longer"
            };
            let start = input.find(strategy).unwrap();
            assert_eq!(
                parse(input, true).unwrap_err(),
                (
                    Span {
                        start,
                        end: start + strategy.len()
                    },
                    GrammarFailureReason::HueInterpolation
                )
            );
        }
        let input = "color-mix(in oklch longer bad, red, blue)";
        let start = input.find("bad").unwrap();
        assert_eq!(
            parse(input, true).unwrap_err(),
            (
                Span {
                    start,
                    end: start + 3
                },
                GrammarFailureReason::Syntax
            )
        );
    }
}
