//! Reached iterative resolution; suspended parents are actual caller-owned slots.
//!
//! Checked syntax is shared by every branch. Only an influencing recipe reads
//! facts or evaluates components. There is no recursive evaluator or source reparse.

use super::channels::{self, Component, Domain};
use super::component_value::{Action as ComponentAction, ComponentValue};
use super::device::{ComputedCmyk, DeviceAction, DeviceResolution};
use super::facts::{CssColorFactIdentity, CssColorFacts};
use super::grammar::{
    FunctionRecipe, GrammarRecord, Interpolation, Leaf, MixRecord, ProfileSpace, RecipeKind,
};
use super::interpolate::{MixFold, WeightSummary, Weights};
use super::keywords::ColorFunction as F;
use super::palette::{PaletteAction, PaletteLookup};
use super::profile::{
    ComponentNameAction, ProfileAction, ProfileComponentLookup, ProfileLookup, ProfileSelection,
};
use super::profile_conversion::{
    ConversionAction, InputAction, ProfileConversionLookup, ProfileInputBuilder,
};
use super::protocol::{CssColorBank, CssColorProtocolError, CssColorSlot, Data};
use super::relative::Binding;
use super::syntax_pass::ValidatedGrammar;
use super::{CssUsedColor, CssUsedColorError};
use crate::{
    CssColorInterpolationSpace as Space, CssPredefinedColorSpace as Predefined,
    CssRelativeColorEnvironment as Environment,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Protocol(CssColorProtocolError),
    Component(super::component_value::Error),
    Channel(channels::Error),
    Numerical(CssUsedColorError),
    Palette(super::palette::PaletteError),
    Profile(super::profile::ProfileError),
    ProfileComponent(super::profile::ComponentNameError),
    Conversion(super::profile_conversion::Error),
    Device(super::device::DeviceError),
    Interpolation(super::interpolate::Error),
    MissingScheme,
    ForeignFacts,
    CustomInterpolationUnsupported,
    ReverseProfileUnsupported,
}
impl From<CssColorProtocolError> for Error {
    fn from(error: CssColorProtocolError) -> Self {
        Self::Protocol(error)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    Begin,
    Origin,
    Wrapper,
    Channels,
    Alpha,
    Finish,
    Profile,
    CustomAlpha,
    CustomInputs,
    MixBuild,
    MixWeights,
    MixColors,
    MixChild,
}
#[derive(Clone, Copy, Debug)]
struct Coordinates {
    origin: Option<CssUsedColor>,
    binding: Option<Binding>,
    components: [Component; 4],
    ordinal: usize,
    alpha: Option<Component>,
}
#[derive(Clone, Copy, Debug)]
struct Custom {
    origin: Option<CssUsedColor>,
    selection: Option<ProfileSelection>,
    alpha: Option<Component>,
}
#[derive(Clone, Copy, Debug)]
struct Wrapper {
    origin: Option<CssUsedColor>,
    ordinal: usize,
}
#[derive(Clone, Copy, Debug)]
struct MixWeights {
    head: Option<usize>,
    next: Option<usize>,
    summary: WeightSummary,
}
#[derive(Clone, Copy, Debug)]
struct MixColors {
    head: Option<usize>,
    next: Option<usize>,
    weights: Weights,
    fold: MixFold,
    mixed_weight: f64,
    member: Option<MixRecord>,
    origin: Option<CssUsedColor>,
}
#[derive(Clone, Copy, Debug)]
enum Continuation {
    Coordinates(Coordinates),
    Custom(Custom),
    Wrapper(Wrapper),
    MixWeights(MixWeights),
    MixColors(MixColors),
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Frame {
    parent: Option<usize>,
    recipe: FunctionRecipe,
    stage: Stage,
    continuation: Continuation,
}
fn same_color(a: Option<CssUsedColor>, b: Option<CssUsedColor>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.identical(b),
        (None, None) => true,
        _ => false,
    }
}
fn same_component(a: Option<Component>, b: Option<Component>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.domain == b.domain && a.value.to_bits() == b.value.to_bits(),
        (None, None) => true,
        _ => false,
    }
}
impl Frame {
    fn new(parent: Option<usize>, recipe: FunctionRecipe) -> Self {
        let continuation = match recipe.function {
            F::LightDark | F::Contrast => Continuation::Wrapper(Wrapper {
                origin: None,
                ordinal: 0,
            }),
            F::Mix => Continuation::MixWeights(MixWeights {
                head: None,
                next: None,
                summary: WeightSummary::default(),
            }),
            _ if matches!(recipe.space, Some(ProfileSpace::Custom { .. })) => {
                Continuation::Custom(Custom {
                    origin: None,
                    selection: None,
                    alpha: None,
                })
            }
            _ => Continuation::Coordinates(Coordinates {
                origin: None,
                binding: None,
                components: [Component::MISSING; 4],
                ordinal: 0,
                alpha: None,
            }),
        };
        Self {
            parent,
            recipe,
            stage: Stage::Begin,
            continuation,
        }
    }
    fn origin(self) -> Option<CssUsedColor> {
        match self.continuation {
            Continuation::Coordinates(v) => v.origin,
            Continuation::Custom(v) => v.origin,
            Continuation::Wrapper(v) => v.origin,
            Continuation::MixColors(v) => v.origin,
            Continuation::MixWeights(_) => None,
        }
    }
    fn set_origin(&mut self, value: Option<CssUsedColor>) -> Result<(), CssColorProtocolError> {
        match &mut self.continuation {
            Continuation::Coordinates(v) => v.origin = value,
            Continuation::Custom(v) => v.origin = value,
            Continuation::Wrapper(v) => v.origin = value,
            Continuation::MixColors(v) => v.origin = value,
            Continuation::MixWeights(_) => return Err(CssColorProtocolError::WrongSlot),
        }
        Ok(())
    }
    fn binding(self) -> Option<Binding> {
        if let Continuation::Coordinates(v) = self.continuation {
            v.binding
        } else {
            None
        }
    }
    fn coordinates(&self) -> Result<&Coordinates, CssColorProtocolError> {
        if let Continuation::Coordinates(v) = &self.continuation {
            Ok(v)
        } else {
            Err(CssColorProtocolError::WrongSlot)
        }
    }
    fn coordinates_mut(&mut self) -> Result<&mut Coordinates, CssColorProtocolError> {
        if let Continuation::Coordinates(v) = &mut self.continuation {
            Ok(v)
        } else {
            Err(CssColorProtocolError::WrongSlot)
        }
    }
    fn custom(&self) -> Result<&Custom, CssColorProtocolError> {
        if let Continuation::Custom(v) = &self.continuation {
            Ok(v)
        } else {
            Err(CssColorProtocolError::WrongSlot)
        }
    }
    fn custom_mut(&mut self) -> Result<&mut Custom, CssColorProtocolError> {
        if let Continuation::Custom(v) = &mut self.continuation {
            Ok(v)
        } else {
            Err(CssColorProtocolError::WrongSlot)
        }
    }
    fn wrapper(&self) -> Result<&Wrapper, CssColorProtocolError> {
        if let Continuation::Wrapper(v) = &self.continuation {
            Ok(v)
        } else {
            Err(CssColorProtocolError::WrongSlot)
        }
    }
    fn wrapper_mut(&mut self) -> Result<&mut Wrapper, CssColorProtocolError> {
        if let Continuation::Wrapper(v) = &mut self.continuation {
            Ok(v)
        } else {
            Err(CssColorProtocolError::WrongSlot)
        }
    }
    fn mix_weights(&self) -> Result<&MixWeights, CssColorProtocolError> {
        if let Continuation::MixWeights(v) = &self.continuation {
            Ok(v)
        } else {
            Err(CssColorProtocolError::WrongSlot)
        }
    }
    fn mix_weights_mut(&mut self) -> Result<&mut MixWeights, CssColorProtocolError> {
        if let Continuation::MixWeights(v) = &mut self.continuation {
            Ok(v)
        } else {
            Err(CssColorProtocolError::WrongSlot)
        }
    }
    fn mix_colors(&self) -> Result<&MixColors, CssColorProtocolError> {
        if let Continuation::MixColors(v) = &self.continuation {
            Ok(v)
        } else {
            Err(CssColorProtocolError::WrongSlot)
        }
    }
    fn mix_colors_mut(&mut self) -> Result<&mut MixColors, CssColorProtocolError> {
        if let Continuation::MixColors(v) = &mut self.continuation {
            Ok(v)
        } else {
            Err(CssColorProtocolError::WrongSlot)
        }
    }
    fn next(self) -> Option<usize> {
        match self.continuation {
            Continuation::MixWeights(v) => v.next,
            Continuation::MixColors(v) => v.next,
            _ => None,
        }
    }
    fn set_next(&mut self, next: Option<usize>) -> Result<(), CssColorProtocolError> {
        match &mut self.continuation {
            Continuation::MixWeights(v) => v.next = next,
            Continuation::MixColors(v) => v.next = next,
            _ => return Err(CssColorProtocolError::WrongSlot),
        }
        Ok(())
    }
    fn identical(self, other: Self) -> bool {
        if self.parent != other.parent || self.recipe != other.recipe || self.stage != other.stage {
            return false;
        }
        match (self.continuation, other.continuation) {
            (Continuation::Coordinates(a), Continuation::Coordinates(b)) => {
                same_color(a.origin, b.origin)
                    && match (a.binding, b.binding) {
                        (Some(a), Some(b)) => a.identical(b),
                        (None, None) => true,
                        _ => false,
                    }
                    && a.components.map(|v| (v.domain, v.value.to_bits()))
                        == b.components.map(|v| (v.domain, v.value.to_bits()))
                    && a.ordinal == b.ordinal
                    && same_component(a.alpha, b.alpha)
            }
            (Continuation::Custom(a), Continuation::Custom(b)) => {
                same_color(a.origin, b.origin)
                    && a.selection == b.selection
                    && same_component(a.alpha, b.alpha)
            }
            (Continuation::Wrapper(a), Continuation::Wrapper(b)) => {
                same_color(a.origin, b.origin) && a.ordinal == b.ordinal
            }
            (Continuation::MixWeights(a), Continuation::MixWeights(b)) => {
                a.head == b.head && a.next == b.next && a.summary.identical(b.summary)
            }
            (Continuation::MixColors(a), Continuation::MixColors(b)) => {
                a.head == b.head
                    && a.next == b.next
                    && (
                        a.weights.omitted.to_bits(),
                        a.weights.scale.to_bits(),
                        a.weights.alpha_multiplier.to_bits(),
                    ) == (
                        b.weights.omitted.to_bits(),
                        b.weights.scale.to_bits(),
                        b.weights.alpha_multiplier.to_bits(),
                    )
                    && a.fold.identical(b.fold)
                    && a.mixed_weight.to_bits() == b.mixed_weight.to_bits()
                    && a.member == b.member
                    && same_color(a.origin, b.origin)
            }
            _ => false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MixTask {
    next: Option<usize>,
    member: MixRecord,
    percentage: Option<f64>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Action {
    ReadRecipe { index: usize },
    ReadParent { index: usize },
    ReadMix { index: usize },
    ReadTask { index: usize },
    ReplaceTask { index: usize },
    AppendFrame { index: usize },
    AppendTask { index: usize },
    Component(ComponentAction),
    Palette(PaletteAction),
    Profile(ProfileAction),
    ReadProfileToken { index: usize },
    ProfileName(ComponentNameAction),
    ReadProfileNameToken { index: usize },
    ProfileInput(InputAction),
    Conversion(ConversionAction),
    Device(DeviceAction),
    ReadScheme,
    Compute,
    Done,
    Stopped,
}
enum State<'owner, 'facts> {
    Recipe(usize),
    Dispatch,
    PendingFrame {
        frame: Frame,
        child: usize,
        index: usize,
    },
    Return(CssUsedColor),
    Parent {
        index: usize,
        value: CssUsedColor,
    },
    Component(ComponentValue<'owner>),
    Palette(PaletteLookup<'facts>),
    ProfileToken {
        index: usize,
        custom_mix: bool,
    },
    Profile {
        cursor: ProfileLookup<'owner, 'facts>,
        custom_mix: bool,
    },
    ProfileNameToken(usize),
    ProfileName(ProfileComponentLookup<'owner, 'facts>),
    CustomBuilder(ProfileInputBuilder<'owner, 'facts>),
    CustomComponent {
        builder: ProfileInputBuilder<'owner, 'facts>,
        cursor: ComponentValue<'owner>,
    },
    Conversion(ProfileConversionLookup<'owner, 'facts>),
    Device(DeviceResolution<'facts>),
    MixRead(usize),
    PendingTask {
        task: MixTask,
        index: usize,
    },
    TaskRead(usize),
    TaskReplace {
        task: MixTask,
        index: usize,
    },
    Scheme,
    Done(CssUsedColor),
    Stopped,
}
pub(crate) struct Resolution<'owner, 'facts> {
    grammar: ValidatedGrammar<'owner>,
    facts: &'facts CssColorFacts<'facts>,
    expected: CssColorFactIdentity,
    frame: Option<Frame>,
    parent: Option<usize>,
    pending_percentage: Option<f64>,
    source: Option<crate::tokenization::color_tokens::Span>,
    state: State<'owner, 'facts>,
}
impl<'owner, 'facts> Resolution<'owner, 'facts> {
    pub(crate) fn start(
        grammar: &ValidatedGrammar<'owner>,
        facts: &'facts CssColorFacts<'facts>,
        expected: CssColorFactIdentity,
    ) -> Self {
        Self {
            grammar: ValidatedGrammar {
                table: grammar.table,
                expression_end: grammar.expression_end,
            },
            facts,
            expected,
            frame: None,
            parent: None,
            pending_percentage: None,
            source: None,
            state: State::Recipe(grammar.table.root),
        }
    }
    pub(crate) fn action(&self) -> Action {
        match &self.state {
            State::Recipe(index) => Action::ReadRecipe { index: *index },
            State::Parent { index, .. } => Action::ReadParent { index: *index },
            State::PendingFrame { index, .. } => Action::AppendFrame { index: *index },
            State::Component(cursor) | State::CustomComponent { cursor, .. } => {
                Action::Component(cursor.action())
            }
            State::Palette(cursor) => Action::Palette(cursor.action()),
            State::ProfileToken { index, .. } => Action::ReadProfileToken { index: *index },
            State::Profile { cursor, .. } => Action::Profile(cursor.action()),
            State::ProfileNameToken(index) => Action::ReadProfileNameToken { index: *index },
            State::ProfileName(cursor) => Action::ProfileName(cursor.action()),
            State::CustomBuilder(cursor) => Action::ProfileInput(cursor.action()),
            State::Conversion(cursor) => Action::Conversion(cursor.action()),
            State::Device(cursor) => Action::Device(cursor.action()),
            State::MixRead(index) => Action::ReadMix { index: *index },
            State::TaskRead(index) => Action::ReadTask { index: *index },
            State::PendingTask { index, .. } => Action::AppendTask { index: *index },
            State::TaskReplace { index, .. } => Action::ReplaceTask { index: *index },
            State::Scheme => Action::ReadScheme,
            State::Done(_) => Action::Done,
            State::Stopped => Action::Stopped,
            State::Dispatch | State::Return(_) => Action::Compute,
        }
    }
    fn fixed_frame(&self) -> Result<Frame, CssColorProtocolError> {
        self.frame.ok_or(CssColorProtocolError::WrongAction)
    }
    fn complete_cylindrical(value: CssUsedColor) -> bool {
        matches!(value.space(), Space::Hsl | Space::Hwb)
            && value.channels().iter().all(Option::is_some)
            && value.alpha().is_some()
    }
    fn return_color(&mut self, value: CssUsedColor) {
        self.state = State::Return(value);
    }
    fn descend(&mut self, child: usize, after: Stage, algorithm_end: usize) -> Result<(), Error> {
        if child < self.grammar.table.start || child >= self.grammar.table.end {
            return Err(CssColorProtocolError::WrongIndex.into());
        }
        algorithm_end
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        let mut frame = self.fixed_frame()?;
        frame.stage = after;
        self.state = State::PendingFrame {
            frame,
            child,
            index: algorithm_end,
        };
        Ok(())
    }
    pub(crate) fn read_recipe(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        if self.action() != (Action::ReadRecipe { index }) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let GrammarRecord::Recipe(recipe) = self.grammar.table.record(index, slot)? else {
            return Err(CssColorProtocolError::WrongSlot.into());
        };
        if index == self.grammar.table.root && self.parent.is_none() {
            self.source = Some(recipe.source);
        }
        match recipe.kind {
            RecipeKind::Leaf(leaf) => match leaf {
                Leaf::Transparent => self.return_color(CssUsedColor::from_srgba8([0; 4])),
                Leaf::Named([r, g, b]) => {
                    self.return_color(CssUsedColor::from_srgba8([r, g, b, 255]))
                }
                Leaf::Hex(bytes) => self.return_color(CssUsedColor::from_srgba8(bytes)),
                Leaf::Current => {
                    self.state = State::Palette(PaletteLookup::current(self.facts, self.expected))
                }
                Leaf::System(keyword) => {
                    self.state =
                        State::Palette(PaletteLookup::system(self.facts, self.expected, keyword))
                }
            },
            RecipeKind::Function(recipe) => {
                self.frame = Some(Frame::new(self.parent, recipe));
                self.state = State::Dispatch;
            }
        }
        Ok(())
    }
    pub(crate) fn read_parent(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        let State::Parent {
            index: expected,
            value,
        } = self.state
        else {
            return Err(CssColorProtocolError::WrongAction.into());
        };
        if index != expected {
            return Err(CssColorProtocolError::WrongIndex.into());
        }
        slot.qualify(
            self.grammar.table.lexical.owner,
            self.grammar.table.lexical.phase,
            index,
            CssColorBank::Algorithm,
        )?;
        let Data::Resolution(frame) = slot.data else {
            return Err(CssColorProtocolError::WrongSlot.into());
        };
        let mut frame = frame;
        match frame.stage {
            Stage::Origin => {
                frame.set_origin(Some(value))?;
                frame.stage = if matches!(frame.recipe.space, Some(ProfileSpace::Custom { .. })) {
                    Stage::Profile
                } else {
                    Stage::Channels
                };
            }
            Stage::Wrapper => {
                frame.set_origin(Some(value))?;
                frame.stage = Stage::Finish;
            }
            Stage::MixChild => {
                frame.set_origin(Some(value))?;
                frame.stage = Stage::MixColors;
            }
            _ => return Err(CssColorProtocolError::WrongSlot.into()),
        }
        self.parent = frame.parent;
        self.frame = Some(frame);
        self.state = State::Dispatch;
        Ok(())
    }
    pub(crate) fn record(&self, index: usize) -> Result<CssColorSlot<'owner>, Error> {
        let data = match self.state {
            State::PendingFrame {
                frame,
                index: expected,
                ..
            } if index == expected => Data::Resolution(frame),
            State::PendingTask {
                task,
                index: expected,
            }
            | State::TaskReplace {
                task,
                index: expected,
            } if index == expected => Data::MixTask(task),
            _ => return Err(CssColorProtocolError::WrongAction.into()),
        };
        Ok(CssColorSlot::new(
            self.grammar.table.lexical.owner,
            self.grammar.table.lexical.phase,
            index,
            data,
        ))
    }
    pub(crate) fn written(&mut self, index: usize, slot: &CssColorSlot<'_>) -> Result<(), Error> {
        slot.qualify(
            self.grammar.table.lexical.owner,
            self.grammar.table.lexical.phase,
            index,
            CssColorBank::Algorithm,
        )?;
        match self.state {
            State::PendingFrame {
                frame,
                child,
                index: expected,
            } if index == expected => {
                let Data::Resolution(actual) = slot.data else {
                    return Err(CssColorProtocolError::WrongSlot.into());
                };
                if !frame.identical(actual) {
                    return Err(CssColorProtocolError::WrongSlot.into());
                }
                self.frame = None;
                self.parent = Some(index);
                self.state = State::Recipe(child);
            }
            State::PendingTask {
                task,
                index: expected,
            } if index == expected => {
                let Data::MixTask(actual) = slot.data else {
                    return Err(CssColorProtocolError::WrongSlot.into());
                };
                if task != actual {
                    return Err(CssColorProtocolError::WrongSlot.into());
                }
                let mut frame = self.fixed_frame()?;
                frame.mix_weights_mut()?.head = Some(index);
                frame.set_next(task.member.previous)?;
                self.frame = Some(frame);
                self.state = if let Some(next) = task.member.previous {
                    State::MixRead(next)
                } else {
                    State::Dispatch
                };
            }
            State::TaskReplace {
                task,
                index: expected,
            } if index == expected => {
                let Data::MixTask(actual) = slot.data else {
                    return Err(CssColorProtocolError::WrongSlot.into());
                };
                if actual != task {
                    return Err(CssColorProtocolError::WrongSlot.into());
                }
                self.frame
                    .as_mut()
                    .ok_or(CssColorProtocolError::WrongAction)?
                    .set_next(task.next)?;
                self.state = State::Dispatch;
            }
            _ => return Err(CssColorProtocolError::WrongAction.into()),
        }
        Ok(())
    }
    pub(crate) fn read_mix(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
        algorithm_end: usize,
    ) -> Result<(), Error> {
        if self.action() != (Action::ReadMix { index }) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let GrammarRecord::Mix(member) = self.grammar.table.record(index, slot)? else {
            return Err(CssColorProtocolError::WrongSlot.into());
        };
        let frame = self.fixed_frame()?;
        if member.ordinal >= frame.recipe.mix_count
            || member.previous.is_some_and(|previous| previous >= index)
        {
            return Err(CssColorProtocolError::WrongSlot.into());
        }
        algorithm_end
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        self.state = State::PendingTask {
            task: MixTask {
                next: frame.mix_weights()?.head,
                member,
                percentage: None,
            },
            index: algorithm_end,
        };
        Ok(())
    }
    fn component(&mut self, index: usize, evaluation_end: usize) -> Result<(), Error> {
        let frame = self.fixed_frame()?;
        self.state = State::Component(ComponentValue::start(
            &self.grammar,
            index,
            evaluation_end,
            frame.binding(),
        )?);
        Ok(())
    }
    pub(crate) fn read_component(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        let cursor = match &mut self.state {
            State::Component(cursor) | State::CustomComponent { cursor, .. } => cursor,
            _ => return Err(CssColorProtocolError::WrongAction.into()),
        };
        cursor.read_component(index, slot).map_err(Error::Component)
    }
    pub(crate) fn component_read(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        let cursor = match &mut self.state {
            State::Component(cursor) | State::CustomComponent { cursor, .. } => cursor,
            _ => return Err(CssColorProtocolError::WrongAction.into()),
        };
        cursor.read(bank, index, slot).map_err(Error::Component)
    }
    pub(crate) fn component_record(&self, index: usize) -> Result<CssColorSlot<'owner>, Error> {
        let cursor = match &self.state {
            State::Component(cursor) | State::CustomComponent { cursor, .. } => cursor,
            _ => return Err(CssColorProtocolError::WrongAction.into()),
        };
        cursor.record(index).map_err(Error::Component)
    }
    pub(crate) fn component_written(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        let cursor = match &mut self.state {
            State::Component(cursor) | State::CustomComponent { cursor, .. } => cursor,
            _ => return Err(CssColorProtocolError::WrongAction.into()),
        };
        cursor.written(index, slot).map_err(Error::Component)
    }
    pub(crate) fn read_profile_token(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        let State::ProfileToken {
            index: expected,
            custom_mix,
        } = self.state
        else {
            return Err(CssColorProtocolError::WrongAction.into());
        };
        if index != expected {
            return Err(CssColorProtocolError::WrongIndex.into());
        }
        let cursor = ProfileLookup::start(
            &self.grammar.table.lexical,
            index,
            slot,
            self.facts,
            self.expected,
        )
        .map_err(Error::Profile)?;
        self.state = State::Profile { cursor, custom_mix };
        Ok(())
    }
    pub(crate) fn profile_source(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        match &mut self.state {
            State::Profile { cursor, .. } => {
                cursor.read_source(index, slot).map_err(Error::Profile)
            }
            State::ProfileName(cursor) => cursor
                .read_source(index, slot)
                .map_err(Error::ProfileComponent),
            _ => Err(CssColorProtocolError::WrongAction.into()),
        }
    }
    pub(crate) fn read_profile_name_token(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        if !matches!(self.state, State::ProfileNameToken(expected) if expected == index) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let selection = self
            .fixed_frame()?
            .custom()?
            .selection
            .ok_or(CssColorProtocolError::WrongSlot)?;
        self.state = State::ProfileName(
            ProfileComponentLookup::start(
                &self.grammar.table.lexical,
                index,
                slot,
                self.facts,
                selection,
            )
            .map_err(Error::ProfileComponent)?,
        );
        Ok(())
    }
    pub(crate) fn profile_input_record(&self, index: usize) -> Result<CssColorSlot<'owner>, Error> {
        let State::CustomBuilder(cursor) = &self.state else {
            return Err(CssColorProtocolError::WrongAction.into());
        };
        cursor.record(index).map_err(Error::Conversion)
    }
    pub(crate) fn profile_input_written(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        let State::CustomBuilder(cursor) = &mut self.state else {
            return Err(CssColorProtocolError::WrongAction.into());
        };
        cursor.appended(index, slot).map_err(Error::Conversion)
    }
    pub(crate) fn conversion_input(
        &mut self,
        index: usize,
        ordinal: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        let State::Conversion(cursor) = &mut self.state else {
            return Err(CssColorProtocolError::WrongAction.into());
        };
        cursor
            .read_input(index, ordinal, slot)
            .map_err(Error::Conversion)
    }
    pub(crate) fn read_scheme(&mut self) -> Result<(), Error> {
        if self.action() != Action::ReadScheme {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let selected = self
            .facts
            .selected_scheme(self.expected)
            .map_err(|error| match error {
                super::facts::SchemeError::ForeignFacts => Error::ForeignFacts,
                super::facts::SchemeError::Unavailable => Error::MissingScheme,
            })?;
        let frame = self
            .frame
            .as_mut()
            .ok_or(CssColorProtocolError::WrongAction)?;
        frame.wrapper_mut()?.ordinal = selected;
        self.state = State::Dispatch;
        Ok(())
    }
    pub(crate) fn read_task(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
        evaluation_end: usize,
    ) -> Result<(), Error> {
        if self.action() != (Action::ReadTask { index }) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        slot.qualify(
            self.grammar.table.lexical.owner,
            self.grammar.table.lexical.phase,
            index,
            CssColorBank::Algorithm,
        )?;
        let Data::MixTask(task) = slot.data else {
            return Err(CssColorProtocolError::WrongSlot.into());
        };
        if let Some(percentage) = self.pending_percentage {
            let mut updated = task;
            updated.percentage = Some(percentage);
            self.pending_percentage = None;
            self.state = State::TaskReplace {
                task: updated,
                index,
            };
            return Ok(());
        }
        let frame = self
            .frame
            .as_mut()
            .ok_or(CssColorProtocolError::WrongAction)?;

        match frame.stage {
            Stage::MixWeights => {
                if let Some(weight) = task.member.weight {
                    self.component(weight, evaluation_end)?;
                } else {
                    frame
                        .mix_weights_mut()?
                        .summary
                        .observe(None)
                        .map_err(Error::Interpolation)?;
                    frame.set_next(task.next)?;
                    self.state = State::Dispatch;
                }
            }
            Stage::MixColors => {
                frame.set_next(task.next)?;
                frame.mix_colors_mut()?.member = Some(task.member);
                frame.mix_colors_mut()?.mixed_weight = frame
                    .mix_colors()?
                    .weights
                    .weight(task.percentage)
                    .map_err(Error::Interpolation)?;
                self.state = State::Dispatch;
            }
            _ => return Err(CssColorProtocolError::WrongSlot.into()),
        }
        Ok(())
    }
    /// Each compute is one fixed kernel/transition. Vector lengths supplied here
    /// are actual admitted observations of the same caller-owned banks.
    pub(crate) fn additional_fact_read(&self) -> Option<super::bounded::CssColorFactRead> {
        use super::bounded::CssColorFactRead as F;
        match &self.state {
            State::Profile {
                cursor,
                custom_mix: true,
            } if cursor.action() == ProfileAction::Done => Some(F::SelectedProfileHeader),
            State::ProfileName(cursor) if cursor.action() == ComponentNameAction::Done => {
                Some(F::SelectedProfileHeader)
            }
            State::Conversion(cursor) if cursor.action() == ConversionAction::Commit => cursor
                .committed_header()
                .map(|ordinal| F::ConversionHeader { ordinal }),
            State::Device(cursor) if cursor.action() == DeviceAction::CommitConversion => {
                Some(F::DeviceHeader)
            }
            _ => None,
        }
    }
    pub(crate) fn kernel(&self) -> super::CssColorKernel {
        use super::CssColorKernel as K;
        match &self.state {
            State::Component(cursor) | State::CustomComponent { cursor, .. } => cursor.kernel(),
            State::Dispatch => match self.frame {
                Some(frame)
                    if frame.stage == Stage::Channels
                        && frame.binding().is_none()
                        && frame.origin().is_some() =>
                {
                    K::ColorConversion
                }
                Some(frame)
                    if frame.stage == Stage::Finish && frame.recipe.function == F::Contrast =>
                {
                    K::Contrast
                }
                Some(frame) if frame.stage == Stage::Finish => K::Channel,
                Some(frame) if frame.stage == Stage::MixColors && frame.origin().is_some() => {
                    if matches!(frame.continuation,Continuation::MixColors(v) if v.fold.has_value())
                    {
                        K::Interpolation
                    } else {
                        K::ColorConversion
                    }
                }
                Some(frame)
                    if frame.stage == Stage::MixWeights && frame.next().is_none()
                        || frame.stage == Stage::MixColors
                            && frame.next().is_none()
                            && matches!(frame.continuation,Continuation::MixColors(v) if v.member.is_none()) =>
                {
                    K::Weight
                }
                _ => K::Transition,
            },
            State::Return(value) if self.parent.is_none() && Self::complete_cylindrical(*value) => {
                K::ColorConversion
            }
            State::TaskRead(_) => K::Weight,
            State::Device(cursor) if cursor.action() == DeviceAction::ComputeFallback => {
                K::ColorConversion
            }
            State::Device(_) => K::Fact,
            State::Palette(_)
            | State::Profile { .. }
            | State::ProfileName(_)
            | State::CustomBuilder(_)
            | State::Conversion(_)
            | State::Scheme => K::Fact,
            _ => K::Transition,
        }
    }
    pub(crate) fn compute(
        &mut self,
        algorithm_end: usize,
        evaluation_end: usize,
    ) -> Result<(), Error> {
        match &mut self.state {
            State::Return(value) => {
                let value = *value;
                self.state = if let Some(index) = self.parent {
                    State::Parent { index, value }
                } else {
                    // Nested origins retain their actual cylindrical coordinates.
                    // Only the final complete used result resolves to sRGB;
                    // missing channels or alpha preserve the natural function.
                    let value = if Self::complete_cylindrical(value) {
                        let converted = value
                            .convert(Space::Predefined(Predefined::Srgb))
                            .map_err(Error::Numerical)?;
                        if value.uses_color_function() {
                            converted.with_color_function()
                        } else {
                            converted.with_legacy_srgb()
                        }
                    } else {
                        value
                    };
                    State::Done(value)
                };
                return Ok(());
            }
            State::Palette(cursor) => {
                if cursor.action() == PaletteAction::Done {
                    let value = cursor.result().map_err(Error::Palette)?.value;
                    self.return_color(value);
                } else {
                    cursor.step(cursor.action()).map_err(Error::Palette)?;
                }
                return Ok(());
            }
            State::Component(cursor) | State::CustomComponent { cursor, .. } => {
                if cursor.action() == ComponentAction::Compute {
                    match cursor.compute() {
                        Ok(()) => return Ok(()),
                        Err(super::component_value::Error::ReverseCustomOrigin { token }) => {
                            cursor.abandon();
                            self.state = State::ProfileNameToken(token);
                            return Ok(());
                        }
                        Err(error) => return Err(Error::Component(error)),
                    }
                }
                if cursor.action() != ComponentAction::Done {
                    return Err(CssColorProtocolError::WrongAction.into());
                }
                let value = cursor.finish()?;
                let state = std::mem::replace(&mut self.state, State::Stopped);
                if let State::CustomComponent {
                    mut builder,
                    mut cursor,
                } = state
                {
                    cursor.abandon();
                    let InputAction::ReadComponent { ordinal } = builder.action() else {
                        return Err(CssColorProtocolError::WrongAction.into());
                    };
                    builder
                        .component(ordinal, value)
                        .map_err(Error::Conversion)?;
                    self.state = State::CustomBuilder(builder);
                    return Ok(());
                }
                let mut frame = self.fixed_frame()?;
                match frame.stage {
                    Stage::Channels => {
                        if frame.coordinates()?.ordinal >= 4 {
                            return Err(CssColorProtocolError::WrongIndex.into());
                        }
                        let coordinates = frame.coordinates_mut()?;
                        coordinates.components[coordinates.ordinal] = value;
                        coordinates.ordinal += 1;
                    }
                    Stage::Alpha | Stage::CustomAlpha => {
                        match &mut frame.continuation {
                            Continuation::Coordinates(v) => v.alpha = Some(value),
                            Continuation::Custom(v) => v.alpha = Some(value),
                            _ => return Err(CssColorProtocolError::WrongSlot.into()),
                        };
                        frame.stage = if frame.stage == Stage::CustomAlpha {
                            Stage::CustomInputs
                        } else {
                            Stage::Finish
                        };
                    }
                    Stage::MixWeights => {
                        if value.domain != Domain::Percentage {
                            return Err(Error::Channel(channels::Error::Grammar));
                        }
                        frame
                            .mix_weights_mut()?
                            .summary
                            .observe(Some(value.value))
                            .map_err(Error::Interpolation)?;
                        let index = frame.next().ok_or(CssColorProtocolError::WrongSlot)?;
                        // The next task read supplies the original fixed next link.
                        self.frame = Some(frame);
                        self.state = State::TaskRead(index);
                        // A second fixed read joins the updated percentage to its actual task.
                        self.pending_percentage = Some(value.value);
                        return Ok(());
                    }
                    _ => return Err(CssColorProtocolError::WrongAction.into()),
                }
                self.frame = Some(frame);
                self.state = State::Dispatch;
                return Ok(());
            }
            State::Profile { cursor, custom_mix } => {
                if cursor.action() == ProfileAction::Done {
                    let selection = cursor.result().map_err(Error::Profile)?;
                    if *custom_mix {
                        return match super::profile_conversion::reverse_custom_channels(
                            self.facts, selection,
                        ) {
                            Err(super::profile_conversion::Error::Unsupported) => {
                                Err(Error::CustomInterpolationUnsupported)
                            }
                            Err(error) => Err(Error::Conversion(error)),
                            Ok(()) => Err(CssColorProtocolError::WrongAction.into()),
                        };
                    }
                    let frame = self
                        .frame
                        .as_mut()
                        .ok_or(CssColorProtocolError::WrongAction)?;
                    frame.custom_mut()?.selection = Some(selection);
                    frame.stage = Stage::CustomAlpha;
                    self.state = State::Dispatch;
                } else {
                    cursor.step(cursor.action()).map_err(Error::Profile)?;
                }
                return Ok(());
            }
            State::ProfileName(cursor) => {
                if cursor.action() == ComponentNameAction::Done {
                    cursor.result().map_err(Error::ProfileComponent)?;
                    return match super::profile_conversion::reverse_custom_channels(
                        self.facts,
                        self.fixed_frame()?
                            .custom()?
                            .selection
                            .ok_or(CssColorProtocolError::WrongSlot)?,
                    ) {
                        Err(super::profile_conversion::Error::Unsupported) => {
                            Err(Error::ReverseProfileUnsupported)
                        }
                        Err(error) => Err(Error::Conversion(error)),
                        Ok(()) => Err(CssColorProtocolError::WrongAction.into()),
                    };
                } else {
                    cursor
                        .step(cursor.action())
                        .map_err(Error::ProfileComponent)?;
                }
                return Ok(());
            }
            State::CustomBuilder(cursor) => {
                match cursor.action() {
                    InputAction::ReadHeader => cursor.read_header().map_err(Error::Conversion)?,
                    InputAction::ReadComponent { ordinal } => {
                        let index = self.fixed_frame()?.recipe.component_index(ordinal)?;
                        let state = std::mem::replace(&mut self.state, State::Stopped);
                        let State::CustomBuilder(builder) = state else {
                            unreachable!()
                        };
                        self.state = State::CustomComponent {
                            builder,
                            cursor: ComponentValue::start(
                                &self.grammar,
                                index,
                                evaluation_end,
                                None,
                            )?,
                        };
                    }
                    InputAction::Done => {
                        let state = std::mem::replace(&mut self.state, State::Stopped);
                        let State::CustomBuilder(builder) = state else {
                            unreachable!()
                        };
                        let inputs = builder.finish().map_err(Error::Conversion)?;
                        self.state = State::Conversion(ProfileConversionLookup::start(
                            &inputs,
                            self.facts,
                            Space::Predefined(Predefined::XyzD50),
                        ));
                    }
                    _ => return Err(CssColorProtocolError::WrongAction.into()),
                }
                return Ok(());
            }
            State::Conversion(cursor) => {
                if cursor.action() == ConversionAction::Done {
                    let value = cursor.result().map_err(Error::Conversion)?.value;
                    self.return_color(value);
                } else {
                    cursor.step(cursor.action()).map_err(Error::Conversion)?;
                }
                return Ok(());
            }
            State::Device(cursor) => {
                if cursor.action() == DeviceAction::Done {
                    let value = cursor.result().map_err(Error::Device)?.value;
                    self.return_color(value);
                } else {
                    cursor.step(cursor.action()).map_err(Error::Device)?;
                }
                return Ok(());
            }
            State::Dispatch => {}
            _ => return Err(CssColorProtocolError::WrongAction.into()),
        }
        let mut frame = self.fixed_frame()?;
        match frame.stage {
            Stage::Begin => {
                match frame.recipe.function {
                    F::LightDark => {
                        frame.stage = Stage::Wrapper;
                        frame.wrapper_mut()?.ordinal = 2;
                        self.frame = Some(frame);
                        self.state = State::Scheme;
                        return Ok(());
                    }
                    F::Contrast => {
                        return self.descend(
                            frame.recipe.children[0].ok_or(CssColorProtocolError::WrongSlot)?,
                            Stage::Wrapper,
                            algorithm_end,
                        );
                    }
                    F::Mix => {
                        if let Some(Interpolation::Custom { token, .. }) =
                            frame.recipe.interpolation
                        {
                            self.state = State::ProfileToken {
                                index: token,
                                custom_mix: true,
                            };
                            return Ok(());
                        }
                        frame.stage = Stage::MixBuild;
                        self.frame = Some(frame);
                        self.state = State::MixRead(
                            frame.recipe.mix.ok_or(CssColorProtocolError::WrongSlot)?,
                        );
                        return Ok(());
                    }
                    _ => {
                        if let Some(origin) = frame.recipe.origin {
                            return self.descend(origin, Stage::Origin, algorithm_end);
                        }
                    }
                }
                frame.stage = if matches!(frame.recipe.space, Some(ProfileSpace::Custom { .. })) {
                    Stage::Profile
                } else {
                    Stage::Channels
                };
            }
            Stage::Wrapper if frame.recipe.function == F::LightDark && frame.origin().is_none() => {
                return self.descend(
                    frame
                        .recipe
                        .children
                        .get(frame.wrapper()?.ordinal)
                        .copied()
                        .flatten()
                        .ok_or(CssColorProtocolError::WrongSlot)?,
                    Stage::Wrapper,
                    algorithm_end,
                );
            }
            Stage::Channels => {
                if frame.binding().is_none()
                    && let Some(origin) = frame.origin()
                {
                    let environment = environment(frame.recipe)?;
                    frame.coordinates_mut()?.binding =
                        Some(Binding::new(environment, origin).map_err(Error::Channel)?);
                    self.frame = Some(frame);
                    return Ok(());
                }
                if frame.coordinates()?.ordinal < frame.recipe.count {
                    let index = frame.recipe.component_index(frame.coordinates()?.ordinal)?;
                    self.frame = Some(frame);
                    return self.component(index, evaluation_end);
                }
                frame.stage = Stage::Alpha;
            }
            Stage::Alpha => {
                if let Some(alpha) = frame.recipe.alpha {
                    self.frame = Some(frame);
                    return self.component(alpha, evaluation_end);
                } else {
                    frame.stage = Stage::Finish;
                }
            }
            Stage::CustomAlpha => {
                if let Some(alpha) = frame.recipe.alpha {
                    self.frame = Some(frame);
                    return self.component(alpha, evaluation_end);
                } else {
                    frame.stage = Stage::CustomInputs;
                }
            }
            Stage::Profile => {
                let Some(ProfileSpace::Custom { token, .. }) = frame.recipe.space else {
                    return Err(CssColorProtocolError::WrongSlot.into());
                };
                self.state = State::ProfileToken {
                    index: token,
                    custom_mix: false,
                };
                return Ok(());
            }
            Stage::CustomInputs => {
                let alpha = if frame.custom()?.alpha.is_some() {
                    channels::alpha(frame.custom()?.alpha, false).map_err(Error::Channel)?
                } else {
                    frame.origin().map_or(Some(1.0), |origin| origin.alpha())
                };
                self.state = State::CustomBuilder(
                    ProfileInputBuilder::start(
                        &self.grammar.table.lexical,
                        self.facts,
                        frame
                            .custom()?
                            .selection
                            .ok_or(CssColorProtocolError::WrongSlot)?,
                        frame.recipe.count,
                        algorithm_end,
                        alpha,
                    )
                    .map_err(Error::Conversion)?,
                );
                return Ok(());
            }
            Stage::Finish => {
                let value = match frame.recipe.function {
                    F::LightDark => frame.origin().ok_or(CssColorProtocolError::WrongSlot)?,
                    F::Contrast => frame
                        .origin()
                        .ok_or(CssColorProtocolError::WrongSlot)?
                        .contrast()
                        .map_err(Error::Numerical)?,
                    F::Alpha => super::relative::alpha_only(
                        frame.origin().ok_or(CssColorProtocolError::WrongSlot)?,
                        frame.coordinates()?.alpha,
                    )
                    .map_err(Error::Channel)?,
                    F::DeviceCmyk => {
                        let computed = ComputedCmyk::new(
                            frame.coordinates()?.components,
                            frame.coordinates()?.alpha,
                            frame.recipe.syntax,
                        )
                        .map_err(Error::Channel)?;
                        self.state = State::Device(DeviceResolution::start(
                            self.facts,
                            self.expected,
                            computed,
                        ));
                        return Ok(());
                    }
                    function => {
                        let predefined = match frame.recipe.space {
                            Some(ProfileSpace::Predefined(space)) => Some(space),
                            _ => None,
                        };
                        let components = [
                            frame.coordinates()?.components[0],
                            frame.coordinates()?.components[1],
                            frame.coordinates()?.components[2],
                        ];
                        if let Some(origin) = frame.origin() {
                            super::relative::coordinates(
                                function,
                                predefined,
                                origin,
                                components,
                                frame.coordinates()?.alpha,
                            )
                            .map_err(Error::Channel)?
                        } else {
                            channels::absolute_coordinates(
                                function,
                                predefined,
                                frame.recipe.syntax,
                                components,
                                frame.coordinates()?.alpha,
                            )
                            .map_err(Error::Channel)?
                        }
                    }
                };
                self.return_color(value);
                return Ok(());
            }
            Stage::MixBuild => {
                frame.stage = Stage::MixWeights;
                frame.mix_weights_mut()?.next = frame.mix_weights()?.head;
            }
            Stage::MixWeights => {
                if let Some(index) = frame.next() {
                    self.state = State::TaskRead(index);
                    return Ok(());
                }
                let weights = frame
                    .mix_weights()?
                    .summary
                    .finish()
                    .map_err(Error::Interpolation)?;
                let method = match frame.recipe.interpolation {
                    Some(Interpolation::Standard(method)) => method,
                    None => crate::CssColorInterpolationMethod::try_new(Space::Oklab, None)
                        .map_err(|_| CssColorProtocolError::WrongSlot)?,
                    _ => return Err(CssColorProtocolError::WrongSlot.into()),
                };
                let head = frame.mix_weights()?.head;
                frame.continuation = Continuation::MixColors(MixColors {
                    head,
                    next: head,
                    weights,
                    fold: MixFold::new(method),
                    mixed_weight: 0.0,
                    member: None,
                    origin: None,
                });
                frame.stage = Stage::MixColors;
            }
            Stage::MixColors => {
                if let Some(value) = frame.mix_colors_mut()?.origin.take() {
                    {
                        let colors = frame.mix_colors_mut()?;
                        colors.fold.observe(value, colors.mixed_weight)
                    }
                    .map_err(Error::Interpolation)?;
                    frame.mix_colors_mut()?.member = None;
                } else if let Some(member) = frame.mix_colors()?.member {
                    self.frame = Some(frame);
                    return self.descend(member.color, Stage::MixChild, algorithm_end);
                } else if let Some(index) = frame.next() {
                    self.frame = Some(frame);
                    self.state = State::TaskRead(index);
                    return Ok(());
                } else {
                    let value = frame
                        .mix_colors()?
                        .fold
                        .finish(frame.mix_colors()?.weights)
                        .map_err(Error::Interpolation)?;
                    self.return_color(value);
                    return Ok(());
                }
            }
            _ => return Err(CssColorProtocolError::WrongAction.into()),
        }
        self.frame = Some(frame);
        self.state = State::Dispatch;
        Ok(())
    }
    pub(crate) fn consumed_source(
        &self,
    ) -> Result<crate::tokenization::color_tokens::Span, CssColorProtocolError> {
        if matches!(self.state, State::Done(_)) {
            self.source.ok_or(CssColorProtocolError::WrongSlot)
        } else {
            Err(CssColorProtocolError::Terminal)
        }
    }
    pub(crate) fn finish(&self) -> Result<CssUsedColor, CssColorProtocolError> {
        if let State::Done(value) = self.state {
            Ok(value)
        } else {
            Err(CssColorProtocolError::Terminal)
        }
    }
    pub(crate) fn abandon(&mut self) {
        match &mut self.state {
            State::Component(cursor) | State::CustomComponent { cursor, .. } => cursor.abandon(),
            State::Palette(cursor) => cursor.abandon(),
            State::Profile { cursor, .. } => cursor.abandon(),
            State::ProfileName(cursor) => cursor.abandon(),
            State::CustomBuilder(cursor) => cursor.abandon(),
            State::Conversion(cursor) => cursor.abandon(),
            State::Device(cursor) => cursor.abandon(),
            _ => {}
        }
        self.state = State::Stopped;
    }
}
fn environment(recipe: FunctionRecipe) -> Result<Environment, CssColorProtocolError> {
    Ok(match recipe.function {
        F::Rgb => Environment::Rgb,
        F::Hsl => Environment::Hsl,
        F::Hwb => Environment::Hwb,
        F::Lab => Environment::Lab,
        F::Lch => Environment::Lch,
        F::Oklab => Environment::Oklab,
        F::Oklch => Environment::Oklch,
        F::Alpha => Environment::Alpha,
        F::Color => match recipe.space {
            Some(ProfileSpace::Predefined(space)) => {
                if matches!(space, Predefined::XyzD50 | Predefined::XyzD65) {
                    Environment::Xyz(space)
                } else {
                    Environment::PredefinedRgb(space)
                }
            }
            _ => return Err(CssColorProtocolError::WrongSlot),
        },
        _ => return Err(CssColorProtocolError::WrongSlot),
    })
}

#[cfg(test)]
mod tests {
    use super::super::facts::{
        CssColorProfileAvailability, CssColorProfileBinding, CssColorProfileConversion,
        CssColorProfileConversionRecord, CssColorPropertyFacts, CssColorSchemeFact,
        CssDeviceColorFact,
    };
    use super::super::protocol::CssColorOperationOwner;
    use super::super::syntax_pass::tests::fixture;
    use super::*;
    fn resolve(input: &str, facts: &CssColorFacts<'_>) -> Result<CssUsedColor, Error> {
        let mut owner = CssColorOperationOwner::new();
        let mut fixture = fixture(&mut owner, input).expect("literal syntax");
        let mut values = Vec::new();
        let mut cursor = Resolution::start(&fixture.grammar, facts, facts.identity);
        // Ordinary test consumer only: original-controller admission belongs to
        // the public guarded action join, not this Vec-owned literal fixture.
        let result = loop {
            let step = match cursor.action() {
                Action::ReadRecipe { index } => cursor.read_recipe(index, &fixture.slots[index]),
                Action::ReadParent { index } => cursor.read_parent(index, &fixture.slots[index]),
                Action::ReadMix { index } => {
                    cursor.read_mix(index, &fixture.slots[index], fixture.slots.len())
                }
                Action::ReadTask { index } => {
                    cursor.read_task(index, &fixture.slots[index], values.len())
                }
                Action::AppendFrame { index } | Action::AppendTask { index } => {
                    assert_eq!(index, fixture.slots.len());
                    fixture.slots.push(cursor.record(index).unwrap());
                    cursor.written(index, &fixture.slots[index])
                }
                Action::ReplaceTask { index } => {
                    fixture.slots[index] = cursor.record(index).unwrap();
                    cursor.written(index, &fixture.slots[index])
                }
                Action::Component(action) => match action {
                    ComponentAction::ReadComponent { index } => {
                        cursor.read_component(index, &fixture.slots[index])
                    }
                    ComponentAction::Read { bank, index } => {
                        let slot = match bank {
                            CssColorBank::Algorithm => &fixture.slots[index],
                            CssColorBank::Arithmetic => &fixture.arithmetic[index],
                            CssColorBank::Evaluation => &values[index],
                        };
                        cursor.component_read(bank, index, slot)
                    }
                    ComponentAction::Append { index } => {
                        assert_eq!(index, values.len());
                        values.push(cursor.component_record(index).unwrap());
                        cursor.component_written(index, &values[index])
                    }
                    ComponentAction::Compute | ComponentAction::Done => {
                        cursor.compute(fixture.slots.len(), values.len())
                    }
                    ComponentAction::Stopped => panic!("component stop"),
                },
                Action::ReadProfileToken { index } => {
                    cursor.read_profile_token(index, &fixture.slots[index])
                }
                Action::Profile(
                    ProfileAction::ReadSource { index } | ProfileAction::ReadPrefix { index },
                ) => cursor.profile_source(index, &fixture.slots[index]),
                Action::ReadProfileNameToken { index } => {
                    cursor.read_profile_name_token(index, &fixture.slots[index])
                }
                Action::ProfileName(ComponentNameAction::ReadSource { index }) => {
                    cursor.profile_source(index, &fixture.slots[index])
                }
                Action::ProfileInput(InputAction::Append { index }) => {
                    assert_eq!(index, fixture.slots.len());
                    fixture
                        .slots
                        .push(cursor.profile_input_record(index).unwrap());
                    cursor.profile_input_written(index, &fixture.slots[index])
                }
                Action::Conversion(ConversionAction::ReadInput { index, ordinal }) => {
                    cursor.conversion_input(index, ordinal, &fixture.slots[index])
                }
                Action::ReadScheme => cursor.read_scheme(),
                Action::Compute
                | Action::Palette(_)
                | Action::Profile(_)
                | Action::ProfileName(_)
                | Action::ProfileInput(_)
                | Action::Conversion(_)
                | Action::Device(_) => cursor.compute(fixture.slots.len(), values.len()),
                Action::Done => break Ok(cursor.finish().unwrap()),
                Action::Stopped => panic!("resolution stop"),
            };
            if let Err(error) = step {
                break Err(error);
            }
        };
        cursor.abandon();
        assert_eq!(cursor.action(), Action::Stopped);
        assert_eq!(cursor.finish(), Err(CssColorProtocolError::Terminal));
        result
    }
    fn facts(scheme: CssColorSchemeFact, device: CssDeviceColorFact<'_>) -> CssColorFacts<'_> {
        CssColorFacts::new(
            CssColorFactIdentity::new(1, 2),
            CssColorPropertyFacts::Unavailable,
            scheme,
            &[],
            &[],
            &[],
            device,
        )
    }
    #[test]
    fn nested_selected_recipes_use_literal_values_without_unreached_fact_access() {
        let facts = facts(CssColorSchemeFact::Light, CssDeviceColorFact::Unknown);
        assert_eq!(
            resolve("light-dark(rgb(calc(10 + 20) 0 0), currentcolor)", &facts)
                .unwrap()
                .channels(),
            [Some(30.0 / 255.0), Some(0.0), Some(0.0)]
        );
        assert_eq!(
            resolve("contrast-color(light-dark(black,currentcolor))", &facts)
                .unwrap()
                .srgb_bytes(),
            Some([255; 3])
        );
        assert_eq!(
            resolve("rgb(from red calc(r / 2) g b / alpha)", &facts)
                .unwrap()
                .channels(),
            [Some(0.5), Some(0.0), Some(0.0)]
        );
        let alpha = resolve("alpha(from red / calc(alpha / 2))", &facts).unwrap();
        assert_eq!(alpha.channels(), [Some(1.0), Some(0.0), Some(0.0)]);
        assert_eq!(alpha.alpha(), Some(0.5));
        let dark = self::facts(CssColorSchemeFact::Dark, CssDeviceColorFact::Unknown);
        assert_eq!(
            resolve("light-dark(currentcolor, blue)", &dark)
                .unwrap()
                .srgb_bytes(),
            Some([0, 0, 255])
        );
        let absent = self::facts(CssColorSchemeFact::Unavailable, CssDeviceColorFact::Unknown);
        assert_eq!(
            resolve("light-dark(red,blue)", &absent),
            Err(Error::MissingScheme)
        );
    }
    #[test]
    fn variadic_mix_preserves_source_order_and_actual_weight_components() {
        let facts = facts(CssColorSchemeFact::Unavailable, CssDeviceColorFact::Unknown);
        let mixed = resolve("color-mix(in srgb, red 20%, blue 30%, black)", &facts).unwrap();
        for (actual, expected) in mixed.channels().into_iter().zip([0.2, 0.0, 0.3]) {
            assert!((actual.unwrap() - expected).abs() < 1e-12);
        }
        assert_eq!(mixed.alpha(), Some(1.0));
        let half = resolve("color-mix(in srgb, red calc(20%), blue 30%)", &facts).unwrap();
        assert!((half.channels()[0].unwrap() - 0.4).abs() < 1e-12);
        assert!((half.channels()[2].unwrap() - 0.6).abs() < 1e-12);
        assert_eq!(half.alpha(), Some(0.5));
    }
    #[test]
    fn all_intrinsic_coordinate_families_reach_the_existing_checked_kernels() {
        let facts = facts(CssColorSchemeFact::Unavailable, CssDeviceColorFact::Absent);
        for (input, space, expected) in [
            (
                "hsl(90 50% 25%)",
                Space::Predefined(Predefined::Srgb),
                [0.25, 0.375, 0.125],
            ),
            ("lab(50% 10 -20)", Space::Lab, [50.0, 10.0, -20.0]),
            ("lch(50 20 90deg)", Space::Lch, [50.0, 20.0, 90.0]),
            ("oklab(50% .1 -.2)", Space::Oklab, [0.5, 0.1, -0.2]),
            ("oklch(.5 .2 90)", Space::Oklch, [0.5, 0.2, 90.0]),
            (
                "color(srgb-linear .1 .2 .3)",
                Space::Predefined(Predefined::SrgbLinear),
                [0.1, 0.2, 0.3],
            ),
        ] {
            let value = resolve(input, &facts).unwrap();
            assert_eq!(value.space(), space);
            assert_eq!(value.channels(), expected.map(Some));
        }
        // Pure hue 90 is [1/2, 1, 0]; HWB applies the remaining 7/10
        // chromatic fraction plus 1/10 white. Only fixed floating rounding is
        // tolerated; the expected RGB values are independent of the converter.
        let value = resolve("hwb(90 10% 20%)", &facts).unwrap();
        assert_eq!(value.space(), Space::Predefined(Predefined::Srgb));
        for (actual, expected) in value.channels().into_iter().zip([0.45, 0.8, 0.1]) {
            assert!((actual.unwrap() - expected).abs() < 1e-15);
        }
        // Color4 16.2.2: missing alpha keeps the same original natural values.
        for (input, space, expected) in [
            ("hsl(90 50% 25% / none)", Space::Hsl, [90.0, 50.0, 25.0]),
            ("hwb(90 10% 20% / none)", Space::Hwb, [90.0, 10.0, 20.0]),
        ] {
            let value = resolve(input, &facts).unwrap();
            assert_eq!(value.space(), space);
            assert_eq!(value.channels(), expected.map(Some));
            assert_eq!(value.alpha(), None);
        }
        let device = resolve("device-cmyk(0 .5 1 .25)", &facts).unwrap();
        assert_eq!(device.channels(), [Some(0.75), Some(0.375), Some(0.0)]);
        let unknown = self::facts(CssColorSchemeFact::Unavailable, CssDeviceColorFact::Unknown);
        assert_eq!(
            resolve("device-cmyk(0 0 0 0)", &unknown),
            Err(Error::Device(
                super::super::device::DeviceError::Unavailable
            ))
        );
    }
    #[test]
    fn custom_forward_conversion_uses_parameter_count_and_qualified_input_before_output() {
        let id = CssColorFactIdentity::new(1, 2);
        let profile = CssColorFactIdentity::new(3, 4);
        let name = crate::CssColorProfileName::try_new("--printer").unwrap();
        let metadata = [CssColorProfileBinding::new(
            &name,
            profile,
            CssColorProfileAvailability::Loaded,
            crate::CssColorProfileRenderingIntent::RelativeColorimetric,
            3,
            &[],
        )];
        let input = [0.2, 0.3, 0.0];
        let expected = CssUsedColor::try_new(
            Space::Predefined(Predefined::XyzD50),
            [Some(0.4), Some(0.5), Some(0.6)],
            Some(1.0),
        )
        .unwrap();
        let conversions = [CssColorProfileConversion::Resolved(
            CssColorProfileConversionRecord::new(
                profile,
                &input,
                crate::CssColorProfileRenderingIntent::RelativeColorimetric,
                Space::Predefined(Predefined::XyzD50),
                expected,
            ),
        )];
        let facts = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &metadata,
            &conversions,
            CssDeviceColorFact::Unknown,
        );
        let actual = resolve("color(--printer .2 calc(.1 + .2) / .5)", &facts);
        // Decimal binary64 addition does not silently qualify a different
        // conversion-input observation.
        assert_eq!(
            actual,
            Err(Error::Conversion(
                super::super::profile_conversion::Error::Unavailable
            ))
        );
        let actual = resolve("color(--printer .2 .3 / .5)", &facts).unwrap();
        assert_eq!(actual.channels(), expected.channels());
        assert_eq!(actual.alpha(), Some(0.5));
    }
    #[test]
    fn exclusive_frame_families_resume_nested_mix_wrapper_and_coordinate_children() {
        let facts = facts(CssColorSchemeFact::Light, CssDeviceColorFact::Absent);
        let text = "color-mix(in srgb, rgb(from light-dark(red,currentcolor) calc(r / 2) g b) 50%, contrast-color(device-cmyk(0 0 0 1)) 50%)";
        let value = resolve(text, &facts).unwrap();
        assert_eq!(value.channels(), [Some(0.75), Some(0.5), Some(0.5)]);
        assert_eq!(value.alpha(), Some(1.0));
        assert!(value.uses_color_function());
    }
}
