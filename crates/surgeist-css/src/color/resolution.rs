//! Reached iterative resolution; suspended parents are actual caller-owned slots.
//!
//! Checked syntax is shared by every branch. Only an influencing recipe reads
//! facts or evaluates components. There is no recursive evaluator or source reparse.

use super::channels::{self, Component, Domain};
use super::component_value::{Action as ComponentAction, ComponentValue};
use super::device::{ComputedCmyk, DeviceAction, DeviceResolution};
use super::grammar::{
    FunctionRecipe, GrammarRecord, Interpolation, Leaf, MixRecord, ProfileSpace, RecipeKind, Site,
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
use super::relative::Binding;
use super::syntax_pass::ValidatedGrammar;
use crate::color::CssRelativeColorEnvironment as Environment;
use crate::color::controlled::protocol::{ColorProtocolError, Data};
use crate::color::controlled::{CssColorBank, CssColorSlot};
use crate::color::{CssColorFactIdentity, CssColorFacts};
use surgeist_color::Color;
use surgeist_color::ColorSpace as Space;
use surgeist_color::PredefinedColorSpace as Predefined;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Protocol(ColorProtocolError),
    Component(super::component_value::Error),
    Channel(channels::Error),
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
impl From<ColorProtocolError> for Error {
    fn from(error: ColorProtocolError) -> Self {
        Self::Protocol(error)
    }
}
// Each variant owns exactly the continuation for that family and phase.
// Suspension stores this same state in a caller-owned algorithm slot.
#[derive(Clone, Copy, Debug)]
enum FrameState {
    CoordinateBegin,
    CoordinateOrigin,
    Channels(Coordinates),
    Alpha(Coordinates),
    Finish(Coordinates),
    ProfileBegin,
    ProfileOrigin,
    Profile(Option<Color>),
    CustomAlpha(Custom),
    CustomInputs(Custom),
    WrapperBegin,
    WrapperScheme,
    WrapperSelected(usize),
    WrapperChild,
    WrapperFinish(Color),
    MixBegin,
    MixBuild(MixWeights),
    MixWeights(MixWeights),
    MixColors(MixColors),
    MixChild { colors: MixColors, weight: f64 },
}
#[derive(Clone, Copy, Debug)]
struct Coordinates {
    origin: Option<Color>,
    binding: Option<Binding>,
    components: [Component; 4],
    ordinal: usize,
    alpha: Option<Component>,
}
impl Coordinates {
    fn new(origin: Option<Color>) -> Self {
        Self {
            origin,
            binding: None,
            components: [Component::MISSING; 4],
            ordinal: 0,
            alpha: None,
        }
    }
    fn identical(self, other: Self) -> bool {
        same_color(self.origin, other.origin)
            && match (self.binding, other.binding) {
                (Some(a), Some(b)) => a.identical(b),
                (None, None) => true,
                _ => false,
            }
            && self.components.map(|v| (v.domain, v.value.to_bits()))
                == other.components.map(|v| (v.domain, v.value.to_bits()))
            && self.ordinal == other.ordinal
            && same_component(self.alpha, other.alpha)
    }
}
#[derive(Clone, Copy, Debug)]
struct Custom {
    origin: Option<Color>,
    selection: ProfileSelection,
    alpha: Option<Component>,
}
#[derive(Clone, Copy, Debug)]
struct MixWeights {
    head: Option<usize>,
    next: Option<usize>,
    summary: WeightSummary,
}
#[derive(Clone, Copy, Debug)]
enum MixCurrent {
    Next,
    Member { member: MixRecord, weight: f64 },
    Returned { value: Color, weight: f64 },
}
#[derive(Clone, Copy, Debug)]
struct MixColors {
    head: Option<usize>,
    next: Option<usize>,
    weights: Weights,
    fold: MixFold,
    current: MixCurrent,
}
impl MixColors {
    fn identical(self, other: Self) -> bool {
        self.head == other.head
            && self.next == other.next
            && (
                self.weights.omitted.to_bits(),
                self.weights.scale.to_bits(),
                self.weights.alpha_multiplier.to_bits(),
            ) == (
                other.weights.omitted.to_bits(),
                other.weights.scale.to_bits(),
                other.weights.alpha_multiplier.to_bits(),
            )
            && self.fold.identical(other.fold)
            && match (self.current, other.current) {
                (MixCurrent::Next, MixCurrent::Next) => true,
                (
                    MixCurrent::Member {
                        member: a,
                        weight: aw,
                    },
                    MixCurrent::Member {
                        member: b,
                        weight: bw,
                    },
                ) => a == b && aw.to_bits() == bw.to_bits(),
                (
                    MixCurrent::Returned {
                        value: a,
                        weight: aw,
                    },
                    MixCurrent::Returned {
                        value: b,
                        weight: bw,
                    },
                ) => a.identical(b) && aw.to_bits() == bw.to_bits(),
                _ => false,
            }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Frame {
    parent: Option<usize>,
    recipe: FunctionRecipe,
    site: Site,
    state: FrameState,
}
fn same_color(a: Option<Color>, b: Option<Color>) -> bool {
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
    fn new(parent: Option<usize>, recipe: FunctionRecipe, site: Site) -> Self {
        let state = match recipe.function {
            F::LightDark | F::Contrast => FrameState::WrapperBegin,
            F::Mix => FrameState::MixBegin,
            _ if matches!(recipe.space, Some(ProfileSpace::Custom { .. })) => {
                FrameState::ProfileBegin
            }
            _ => FrameState::CoordinateBegin,
        };
        Self {
            parent,
            recipe,
            site,
            state,
        }
    }
    fn binding(self) -> Option<Binding> {
        match self.state {
            FrameState::Channels(v) | FrameState::Alpha(v) | FrameState::Finish(v) => v.binding,
            _ => None,
        }
    }
    fn custom(&self) -> Result<&Custom, ColorProtocolError> {
        match &self.state {
            FrameState::CustomAlpha(v) | FrameState::CustomInputs(v) => Ok(v),
            _ => Err(ColorProtocolError::WrongSlot),
        }
    }
    fn mix_weights(&self) -> Result<&MixWeights, ColorProtocolError> {
        match &self.state {
            FrameState::MixBuild(v) | FrameState::MixWeights(v) => Ok(v),
            _ => Err(ColorProtocolError::WrongSlot),
        }
    }
    fn mix_weights_mut(&mut self) -> Result<&mut MixWeights, ColorProtocolError> {
        match &mut self.state {
            FrameState::MixBuild(v) | FrameState::MixWeights(v) => Ok(v),
            _ => Err(ColorProtocolError::WrongSlot),
        }
    }
    fn set_next(&mut self, next: Option<usize>) -> Result<(), ColorProtocolError> {
        match &mut self.state {
            FrameState::MixBuild(v) | FrameState::MixWeights(v) => v.next = next,
            FrameState::MixColors(v) => v.next = next,
            _ => return Err(ColorProtocolError::WrongSlot),
        }
        Ok(())
    }
    fn identical(self, other: Self) -> bool {
        if self.parent != other.parent || self.recipe != other.recipe || self.site != other.site {
            return false;
        }
        match (self.state, other.state) {
            (FrameState::CoordinateBegin, FrameState::CoordinateBegin)
            | (FrameState::CoordinateOrigin, FrameState::CoordinateOrigin)
            | (FrameState::ProfileBegin, FrameState::ProfileBegin)
            | (FrameState::ProfileOrigin, FrameState::ProfileOrigin)
            | (FrameState::WrapperBegin, FrameState::WrapperBegin)
            | (FrameState::WrapperScheme, FrameState::WrapperScheme)
            | (FrameState::WrapperChild, FrameState::WrapperChild)
            | (FrameState::MixBegin, FrameState::MixBegin) => true,
            (FrameState::Channels(a), FrameState::Channels(b))
            | (FrameState::Alpha(a), FrameState::Alpha(b))
            | (FrameState::Finish(a), FrameState::Finish(b)) => a.identical(b),
            (FrameState::Profile(a), FrameState::Profile(b)) => same_color(a, b),
            (FrameState::CustomAlpha(a), FrameState::CustomAlpha(b))
            | (FrameState::CustomInputs(a), FrameState::CustomInputs(b)) => {
                same_color(a.origin, b.origin)
                    && a.selection == b.selection
                    && same_component(a.alpha, b.alpha)
            }
            (FrameState::WrapperSelected(a), FrameState::WrapperSelected(b)) => a == b,
            (FrameState::WrapperFinish(a), FrameState::WrapperFinish(b)) => a.identical(b),
            (FrameState::MixBuild(a), FrameState::MixBuild(b))
            | (FrameState::MixWeights(a), FrameState::MixWeights(b)) => {
                a.head == b.head && a.next == b.next && a.summary.identical(b.summary)
            }
            (FrameState::MixColors(a), FrameState::MixColors(b)) => a.identical(b),
            (
                FrameState::MixChild {
                    colors: a,
                    weight: aw,
                },
                FrameState::MixChild {
                    colors: b,
                    weight: bw,
                },
            ) => a.identical(b) && aw.to_bits() == bw.to_bits(),
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
    Return(Color),
    Parent {
        index: usize,
        value: Color,
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
    Done(Color),
    Stopped,
}
pub(crate) struct Resolution<'owner, 'facts> {
    grammar: ValidatedGrammar<'owner>,
    facts: &'facts CssColorFacts<'facts>,
    expected: CssColorFactIdentity,
    frame: Option<Frame>,
    parent: Option<usize>,
    pending_percentage: Option<f64>,
    source: Option<Site>,
    reached: Option<Site>,
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
            reached: None,
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
    fn fixed_frame(&self) -> Result<Frame, ColorProtocolError> {
        self.frame.ok_or(ColorProtocolError::WrongAction)
    }
    fn complete_cylindrical(value: Color) -> bool {
        matches!(value.space(), Space::Hsl | Space::Hwb)
            && value.channels().iter().all(Option::is_some)
            && value.alpha().is_some()
    }
    fn return_color(&mut self, value: Color) {
        self.state = State::Return(value);
    }
    fn descend(
        &mut self,
        child: usize,
        after: FrameState,
        algorithm_end: usize,
    ) -> Result<(), Error> {
        if child < self.grammar.table.start || child >= self.grammar.table.end {
            return Err(ColorProtocolError::WrongIndex.into());
        }
        algorithm_end
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        let mut frame = self.fixed_frame()?;
        frame.state = after;
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
            return Err(ColorProtocolError::WrongAction.into());
        }
        let GrammarRecord::Recipe(recipe) = self.grammar.table.record(index, slot)? else {
            return Err(ColorProtocolError::WrongSlot.into());
        };
        if index == self.grammar.table.root && self.parent.is_none() {
            self.source = Some(recipe.site);
        }
        let recipe_site = recipe.site;
        self.reached = Some(recipe_site);
        match recipe.kind {
            RecipeKind::Leaf(leaf) => match leaf {
                Leaf::Transparent => self.return_color(Color::from_srgba8([0; 4])),
                Leaf::Named([r, g, b]) => self.return_color(Color::from_srgba8([r, g, b, 255])),
                Leaf::Hex(bytes) => self.return_color(Color::from_srgba8(bytes)),
                Leaf::Current => {
                    self.state = State::Palette(PaletteLookup::current(self.facts, self.expected))
                }
                Leaf::System(keyword) => {
                    self.state =
                        State::Palette(PaletteLookup::system(self.facts, self.expected, keyword))
                }
            },
            RecipeKind::Function(recipe) => {
                self.frame = Some(Frame::new(self.parent, recipe, recipe_site));
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
            return Err(ColorProtocolError::WrongAction.into());
        };
        if index != expected {
            return Err(ColorProtocolError::WrongIndex.into());
        }
        slot.qualify(
            self.grammar.table.input.records().owner,
            self.grammar.table.input.records().phase,
            index,
            CssColorBank::Algorithm,
        )?;
        let Data::Resolution(frame) = slot.data else {
            return Err(ColorProtocolError::WrongSlot.into());
        };
        let mut frame = frame;
        frame.state = match frame.state {
            FrameState::CoordinateOrigin => FrameState::Channels(Coordinates::new(Some(value))),
            FrameState::ProfileOrigin => FrameState::Profile(Some(value)),
            FrameState::WrapperChild => FrameState::WrapperFinish(value),
            FrameState::MixChild { mut colors, weight } => {
                colors.current = MixCurrent::Returned { value, weight };
                FrameState::MixColors(colors)
            }
            _ => return Err(ColorProtocolError::WrongSlot.into()),
        };
        self.reached = Some(frame.site);
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
            _ => return Err(ColorProtocolError::WrongAction.into()),
        };
        Ok(CssColorSlot::new(
            self.grammar.table.input.records().owner,
            self.grammar.table.input.records().phase,
            index,
            data,
        ))
    }
    pub(crate) fn written(&mut self, index: usize, slot: &CssColorSlot<'_>) -> Result<(), Error> {
        slot.qualify(
            self.grammar.table.input.records().owner,
            self.grammar.table.input.records().phase,
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
                    return Err(ColorProtocolError::WrongSlot.into());
                };
                if !frame.identical(actual) {
                    return Err(ColorProtocolError::WrongSlot.into());
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
                    return Err(ColorProtocolError::WrongSlot.into());
                };
                if task != actual {
                    return Err(ColorProtocolError::WrongSlot.into());
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
                    return Err(ColorProtocolError::WrongSlot.into());
                };
                if actual != task {
                    return Err(ColorProtocolError::WrongSlot.into());
                }
                self.frame
                    .as_mut()
                    .ok_or(ColorProtocolError::WrongAction)?
                    .set_next(task.next)?;
                self.state = State::Dispatch;
            }
            _ => return Err(ColorProtocolError::WrongAction.into()),
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
            return Err(ColorProtocolError::WrongAction.into());
        }
        let GrammarRecord::Mix(member) = self.grammar.table.record(index, slot)? else {
            return Err(ColorProtocolError::WrongSlot.into());
        };
        let frame = self.fixed_frame()?;
        if member.ordinal >= frame.recipe.mix_count
            || member.previous.is_some_and(|previous| previous >= index)
        {
            return Err(ColorProtocolError::WrongSlot.into());
        }
        algorithm_end
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
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
        if matches!(frame.site, Site::Authored { .. }) {
            self.reached = Some(frame.site);
        }
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
            _ => return Err(ColorProtocolError::WrongAction.into()),
        };
        cursor
            .read_component(index, slot)
            .map_err(Error::Component)?;
        if let Data::Grammar(GrammarRecord::Component(record)) = slot.data
            && let super::grammar::RawComponent::Authored { site } = record.component
        {
            self.reached = Some(Site::Authored { record: site });
        }
        Ok(())
    }
    pub(crate) fn component_read(
        &mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        let cursor = match &mut self.state {
            State::Component(cursor) | State::CustomComponent { cursor, .. } => cursor,
            _ => return Err(ColorProtocolError::WrongAction.into()),
        };
        cursor.read(bank, index, slot).map_err(Error::Component)?;
        if let Some(record) = cursor.authored_site() {
            self.reached = Some(Site::Authored { record });
        }
        Ok(())
    }
    pub(crate) fn component_record(&self, index: usize) -> Result<CssColorSlot<'owner>, Error> {
        let cursor = match &self.state {
            State::Component(cursor) | State::CustomComponent { cursor, .. } => cursor,
            _ => return Err(ColorProtocolError::WrongAction.into()),
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
            _ => return Err(ColorProtocolError::WrongAction.into()),
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
            return Err(ColorProtocolError::WrongAction.into());
        };
        if index != expected {
            return Err(ColorProtocolError::WrongIndex.into());
        }
        let cursor = ProfileLookup::start(
            &self.grammar.table.input.records(),
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
            _ => Err(ColorProtocolError::WrongAction.into()),
        }
    }
    pub(crate) fn read_profile_name_token(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        if !matches!(self.state, State::ProfileNameToken(expected) if expected == index) {
            return Err(ColorProtocolError::WrongAction.into());
        }
        let selection = self.fixed_frame()?.custom()?.selection;
        self.state = State::ProfileName(
            ProfileComponentLookup::start(
                &self.grammar.table.input.records(),
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
            return Err(ColorProtocolError::WrongAction.into());
        };
        cursor.record(index).map_err(Error::Conversion)
    }
    pub(crate) fn profile_input_written(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), Error> {
        let State::CustomBuilder(cursor) = &mut self.state else {
            return Err(ColorProtocolError::WrongAction.into());
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
            return Err(ColorProtocolError::WrongAction.into());
        };
        cursor
            .read_input(index, ordinal, slot)
            .map_err(Error::Conversion)
    }
    pub(crate) fn read_scheme(&mut self) -> Result<(), Error> {
        if self.action() != Action::ReadScheme {
            return Err(ColorProtocolError::WrongAction.into());
        }
        let selected = self
            .facts
            .selected_scheme(self.expected)
            .map_err(|error| match error {
                super::facts::SchemeError::ForeignFacts => Error::ForeignFacts,
                super::facts::SchemeError::Unavailable => Error::MissingScheme,
            })?;
        let frame = self.frame.as_mut().ok_or(ColorProtocolError::WrongAction)?;
        if !matches!(frame.state, FrameState::WrapperScheme) {
            return Err(ColorProtocolError::WrongSlot.into());
        }
        frame.state = FrameState::WrapperSelected(selected);
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
            return Err(ColorProtocolError::WrongAction.into());
        }
        slot.qualify(
            self.grammar.table.input.records().owner,
            self.grammar.table.input.records().phase,
            index,
            CssColorBank::Algorithm,
        )?;
        let Data::MixTask(task) = slot.data else {
            return Err(ColorProtocolError::WrongSlot.into());
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
        let frame = self.frame.as_mut().ok_or(ColorProtocolError::WrongAction)?;

        match &mut frame.state {
            FrameState::MixWeights(weights) => {
                if let Some(weight) = task.member.weight {
                    self.component(weight, evaluation_end)?;
                } else {
                    weights
                        .summary
                        .observe(None)
                        .map_err(Error::Interpolation)?;
                    weights.next = task.next;
                    self.state = State::Dispatch;
                }
            }
            FrameState::MixColors(colors) => {
                colors.next = task.next;
                colors.current = MixCurrent::Member {
                    member: task.member,
                    weight: colors
                        .weights
                        .weight(task.percentage)
                        .map_err(Error::Interpolation)?,
                };
                self.state = State::Dispatch;
            }
            _ => return Err(ColorProtocolError::WrongSlot.into()),
        }
        Ok(())
    }
    /// Each compute is one fixed kernel/transition. Vector lengths supplied here
    /// are actual admitted observations of the same caller-owned banks.
    pub(crate) fn additional_fact_read(
        &self,
    ) -> Option<crate::color::controlled::CssColorFactRead> {
        use crate::color::controlled::CssColorFactRead as F;
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
    pub(crate) fn kernel(&self) -> crate::color::controlled::CssColorKernel {
        use crate::color::controlled::CssColorKernel as K;
        match &self.state {
            State::Component(cursor) | State::CustomComponent { cursor, .. } => cursor.kernel(),
            State::Dispatch => match self.frame.map(|frame| (frame.recipe.function, frame.state)) {
                Some((_, FrameState::Channels(v))) if v.binding.is_none() && v.origin.is_some() => {
                    K::ColorConversion
                }
                Some((F::Contrast, FrameState::WrapperFinish(_))) => K::Contrast,
                Some((_, FrameState::Finish(_) | FrameState::WrapperFinish(_))) => K::Channel,
                Some((_, FrameState::MixColors(v)))
                    if matches!(v.current, MixCurrent::Returned { .. }) =>
                {
                    if v.fold.has_value() {
                        K::Interpolation
                    } else {
                        K::ColorConversion
                    }
                }
                Some((_, FrameState::MixWeights(v))) if v.next.is_none() => K::Weight,
                Some((_, FrameState::MixColors(v)))
                    if v.next.is_none() && matches!(v.current, MixCurrent::Next) =>
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
                        if value.serialization_family()
                            == surgeist_color::SerializationFamily::ColorFunction
                        {
                            value
                                .converted(Space::Predefined(Predefined::Srgb))
                                .with_color_function()
                        } else {
                            value.to_legacy_srgb()
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
                    return Err(ColorProtocolError::WrongAction.into());
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
                        return Err(ColorProtocolError::WrongAction.into());
                    };
                    builder
                        .component(ordinal, value)
                        .map_err(Error::Conversion)?;
                    if let Some(frame) = self.frame
                        && matches!(frame.site, Site::Authored { .. })
                    {
                        self.reached = Some(frame.site);
                    }
                    self.state = State::CustomBuilder(builder);
                    return Ok(());
                }
                let mut frame = self.fixed_frame()?;
                match &mut frame.state {
                    FrameState::Channels(coordinates) => {
                        if coordinates.ordinal >= 4 {
                            return Err(ColorProtocolError::WrongIndex.into());
                        }
                        coordinates.components[coordinates.ordinal] = value;
                        coordinates.ordinal += 1;
                    }
                    FrameState::Alpha(coordinates) => {
                        coordinates.alpha = Some(value);
                        frame.state = FrameState::Finish(*coordinates);
                    }
                    FrameState::CustomAlpha(custom) => {
                        custom.alpha = Some(value);
                        frame.state = FrameState::CustomInputs(*custom);
                    }
                    FrameState::MixWeights(weights) => {
                        if value.domain != Domain::Percentage {
                            return Err(Error::Channel(channels::Error::Grammar));
                        }
                        weights
                            .summary
                            .observe(Some(value.value))
                            .map_err(Error::Interpolation)?;
                        let index = weights.next.ok_or(ColorProtocolError::WrongSlot)?;
                        // A second fixed read joins the percentage to its actual task.
                        self.frame = Some(frame);
                        self.state = State::TaskRead(index);
                        self.pending_percentage = Some(value.value);
                        return Ok(());
                    }
                    _ => return Err(ColorProtocolError::WrongAction.into()),
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
                            Ok(()) => Err(ColorProtocolError::WrongAction.into()),
                        };
                    }
                    let frame = self.frame.as_mut().ok_or(ColorProtocolError::WrongAction)?;
                    let FrameState::Profile(origin) = frame.state else {
                        return Err(ColorProtocolError::WrongSlot.into());
                    };
                    frame.state = FrameState::CustomAlpha(Custom {
                        origin,
                        selection,
                        alpha: None,
                    });
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
                        self.fixed_frame()?.custom()?.selection,
                    ) {
                        Err(super::profile_conversion::Error::Unsupported) => {
                            Err(Error::ReverseProfileUnsupported)
                        }
                        Err(error) => Err(Error::Conversion(error)),
                        Ok(()) => Err(ColorProtocolError::WrongAction.into()),
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
                        if let Some(frame) = self.frame
                            && matches!(frame.site, Site::Authored { .. })
                        {
                            self.reached = Some(frame.site);
                        }
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
                    _ => return Err(ColorProtocolError::WrongAction.into()),
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
            _ => return Err(ColorProtocolError::WrongAction.into()),
        }
        let mut frame = self.fixed_frame()?;
        if matches!(frame.site, Site::Authored { .. }) {
            self.reached = Some(frame.site);
        }
        match frame.state {
            FrameState::CoordinateBegin => {
                if let Some(origin) = frame.recipe.origin {
                    return self.descend(origin, FrameState::CoordinateOrigin, algorithm_end);
                }
                frame.state = FrameState::Channels(Coordinates::new(None));
            }
            FrameState::ProfileBegin => {
                if let Some(origin) = frame.recipe.origin {
                    return self.descend(origin, FrameState::ProfileOrigin, algorithm_end);
                }
                frame.state = FrameState::Profile(None);
            }
            FrameState::WrapperBegin => match frame.recipe.function {
                F::LightDark => {
                    frame.state = FrameState::WrapperScheme;
                    self.frame = Some(frame);
                    self.state = State::Scheme;
                    return Ok(());
                }
                F::Contrast => {
                    return self.descend(
                        frame.recipe.children[0].ok_or(ColorProtocolError::WrongSlot)?,
                        FrameState::WrapperChild,
                        algorithm_end,
                    );
                }
                _ => return Err(ColorProtocolError::WrongSlot.into()),
            },
            FrameState::WrapperSelected(ordinal) => {
                return self.descend(
                    frame
                        .recipe
                        .children
                        .get(ordinal)
                        .copied()
                        .flatten()
                        .ok_or(ColorProtocolError::WrongSlot)?,
                    FrameState::WrapperChild,
                    algorithm_end,
                );
            }
            FrameState::WrapperFinish(origin) => {
                let value = if frame.recipe.function == F::Contrast {
                    origin.black_white_contrast()
                } else {
                    origin
                };
                self.return_color(value);
                return Ok(());
            }
            FrameState::Channels(mut coordinates) => {
                if coordinates.binding.is_none()
                    && let Some(origin) = coordinates.origin
                {
                    coordinates.binding = Some(
                        Binding::new(environment(frame.recipe)?, origin).map_err(Error::Channel)?,
                    );
                    frame.state = FrameState::Channels(coordinates);
                    self.frame = Some(frame);
                    return Ok(());
                }
                if coordinates.ordinal < frame.recipe.count {
                    let index = frame.recipe.component_index(coordinates.ordinal)?;
                    self.frame = Some(frame);
                    return self.component(index, evaluation_end);
                }
                frame.state = FrameState::Alpha(coordinates);
            }
            FrameState::Alpha(coordinates) => {
                if let Some(alpha) = frame.recipe.alpha {
                    self.frame = Some(frame);
                    return self.component(alpha, evaluation_end);
                }
                frame.state = FrameState::Finish(coordinates);
            }
            FrameState::CustomAlpha(custom) => {
                if let Some(alpha) = frame.recipe.alpha {
                    self.frame = Some(frame);
                    return self.component(alpha, evaluation_end);
                }
                frame.state = FrameState::CustomInputs(custom);
            }
            FrameState::Profile(_) => {
                let Some(ProfileSpace::Custom { token, .. }) = frame.recipe.space else {
                    return Err(ColorProtocolError::WrongSlot.into());
                };
                self.state = State::ProfileToken {
                    index: token,
                    custom_mix: false,
                };
                return Ok(());
            }
            FrameState::CustomInputs(custom) => {
                let alpha = if custom.alpha.is_some() {
                    channels::alpha(custom.alpha, false).map_err(Error::Channel)?
                } else {
                    custom
                        .origin
                        .map_or(surgeist_color::UnitInterval::new(1.0), |origin| {
                            origin.unit_alpha()
                        })
                };
                self.state = State::CustomBuilder(
                    ProfileInputBuilder::start(
                        &self.grammar.table.input.records(),
                        self.facts,
                        custom.selection,
                        frame.recipe.count,
                        algorithm_end,
                        alpha,
                    )
                    .map_err(Error::Conversion)?,
                );
                return Ok(());
            }
            FrameState::Finish(coordinates) => {
                let value = match frame.recipe.function {
                    F::Alpha => super::relative::alpha_only(
                        coordinates.origin.ok_or(ColorProtocolError::WrongSlot)?,
                        coordinates.alpha,
                    )
                    .map_err(Error::Channel)?,
                    F::DeviceCmyk => {
                        let computed = ComputedCmyk::new(
                            coordinates.components,
                            coordinates.alpha,
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
                            coordinates.components[0],
                            coordinates.components[1],
                            coordinates.components[2],
                        ];
                        if let Some(origin) = coordinates.origin {
                            super::relative::coordinates(
                                function,
                                predefined,
                                origin,
                                components,
                                coordinates.alpha,
                            )
                            .map_err(Error::Channel)?
                        } else {
                            channels::absolute_coordinates(
                                function,
                                predefined,
                                frame.recipe.syntax,
                                components,
                                coordinates.alpha,
                            )
                            .map_err(Error::Channel)?
                        }
                    }
                };
                self.return_color(value);
                return Ok(());
            }
            FrameState::MixBegin => {
                if let Some(Interpolation::Custom { token, .. }) = frame.recipe.interpolation {
                    self.state = State::ProfileToken {
                        index: token,
                        custom_mix: true,
                    };
                    return Ok(());
                }
                frame.state = FrameState::MixBuild(MixWeights {
                    head: None,
                    next: None,
                    summary: WeightSummary::default(),
                });
                self.frame = Some(frame);
                self.state = State::MixRead(frame.recipe.mix.ok_or(ColorProtocolError::WrongSlot)?);
                return Ok(());
            }
            FrameState::MixBuild(mut weights) => {
                weights.next = weights.head;
                frame.state = FrameState::MixWeights(weights);
            }
            FrameState::MixWeights(weights) => {
                if let Some(index) = weights.next {
                    self.state = State::TaskRead(index);
                    return Ok(());
                }
                let resolved = weights.summary.finish().map_err(Error::Interpolation)?;
                let method = match frame.recipe.interpolation {
                    Some(Interpolation::Standard(method)) => method,
                    None => crate::color::CssColorInterpolationMethod::try_new(Space::Oklab, None)
                        .map_err(|_| ColorProtocolError::WrongSlot)?,
                    _ => return Err(ColorProtocolError::WrongSlot.into()),
                };
                frame.state = FrameState::MixColors(MixColors {
                    head: weights.head,
                    next: weights.head,
                    weights: resolved,
                    fold: MixFold::new(method),
                    current: MixCurrent::Next,
                });
            }
            FrameState::MixColors(mut colors) => match colors.current {
                MixCurrent::Returned { value, weight } => {
                    colors
                        .fold
                        .observe(value, weight)
                        .map_err(Error::Interpolation)?;
                    colors.current = MixCurrent::Next;
                    frame.state = FrameState::MixColors(colors);
                }
                MixCurrent::Member { member, weight } => {
                    colors.current = MixCurrent::Next;
                    return self.descend(
                        member.color,
                        FrameState::MixChild { colors, weight },
                        algorithm_end,
                    );
                }
                MixCurrent::Next => {
                    if let Some(index) = colors.next {
                        self.state = State::TaskRead(index);
                    } else {
                        let value = colors
                            .fold
                            .finish(colors.weights)
                            .map_err(Error::Interpolation)?;
                        self.return_color(value);
                    }
                    return Ok(());
                }
            },
            _ => return Err(ColorProtocolError::WrongAction.into()),
        }
        self.frame = Some(frame);
        self.state = State::Dispatch;
        Ok(())
    }
    pub(crate) fn reached_site(&self) -> Result<Site, ColorProtocolError> {
        if matches!(self.state, State::Dispatch) {
            self.frame
                .map(|frame| frame.site)
                .ok_or(ColorProtocolError::WrongSlot)
        } else {
            self.reached.ok_or(ColorProtocolError::WrongSlot)
        }
    }
    pub(crate) fn consumed_site(&self) -> Result<Site, ColorProtocolError> {
        if matches!(self.state, State::Done(_)) {
            self.source.ok_or(ColorProtocolError::WrongSlot)
        } else {
            Err(ColorProtocolError::Terminal)
        }
    }
    pub(crate) fn consumed_source(
        &self,
    ) -> Result<crate::tokenization::tokenizer::Span, ColorProtocolError> {
        if matches!(self.state, State::Done(_)) {
            self.source.ok_or(ColorProtocolError::WrongSlot)?.source()
        } else {
            Err(ColorProtocolError::Terminal)
        }
    }
    pub(crate) fn finish(&self) -> Result<Color, ColorProtocolError> {
        if let State::Done(value) = self.state {
            Ok(value)
        } else {
            Err(ColorProtocolError::Terminal)
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
fn environment(recipe: FunctionRecipe) -> Result<Environment, ColorProtocolError> {
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
            _ => return Err(ColorProtocolError::WrongSlot),
        },
        _ => return Err(ColorProtocolError::WrongSlot),
    })
}

#[cfg(test)]
mod tests {
    use super::super::syntax_pass::tests::fixture;
    use super::*;
    use crate::color::controlled::CssColorOperationOwner;
    use crate::color::{
        CssColorProfileAvailability, CssColorProfileBinding, CssColorProfileConversion,
        CssColorProfileConversionRecord, CssColorPropertyFacts, CssColorSchemeFact,
        CssDeviceColorFact,
    };
    fn resolve(input: &str, facts: &CssColorFacts<'_>) -> Result<Color, Error> {
        resolve_suspending(input, facts, None)
    }
    fn resolve_suspending(
        input: &str,
        facts: &CssColorFacts<'_>,
        cancel_at: Option<usize>,
    ) -> Result<Color, Error> {
        let foreign = CssColorOperationOwner::new();
        let mut suspensions = 0;
        let mut owner = CssColorOperationOwner::new();
        let mut fixture = fixture(&mut owner, input).expect("literal syntax");
        let mut values = Vec::new();
        let mut cursor = Resolution::start(&fixture.grammar, facts, facts.identity);
        // Ordinary test consumer only: original-controller admission belongs to
        // the public guarded action join, not this Vec-owned literal fixture.
        let result = loop {
            let step = match cursor.action() {
                Action::ReadRecipe { index } => cursor.read_recipe(index, &fixture.slots[index]),
                Action::ReadParent { index } => {
                    let pending = cursor.action();
                    let record = &fixture.slots[index];
                    let foreign_record =
                        CssColorSlot::new(&foreign, record.phase, index, record.data);
                    assert_eq!(
                        cursor.read_parent(index, &foreign_record),
                        Err(Error::Protocol(ColorProtocolError::ForeignSlot))
                    );
                    assert_eq!(cursor.action(), pending);
                    let wrong =
                        CssColorSlot::new(record.owner, record.phase, index, Data::NameUnit('x'));
                    assert_eq!(
                        cursor.read_parent(index, &wrong),
                        Err(Error::Protocol(ColorProtocolError::WrongSlot))
                    );
                    assert_eq!(cursor.action(), pending);
                    assert_eq!(
                        cursor.read_parent(index + 1, record),
                        Err(Error::Protocol(ColorProtocolError::WrongIndex))
                    );
                    assert_eq!(cursor.action(), pending);
                    if cancel_at == Some(suspensions) {
                        cursor.abandon();
                        break Err(Error::Protocol(ColorProtocolError::Terminal));
                    }
                    suspensions += 1;
                    cursor.read_parent(index, record)
                }
                Action::ReadMix { index } => {
                    cursor.read_mix(index, &fixture.slots[index], fixture.slots.len())
                }
                Action::ReadTask { index } => {
                    cursor.read_task(index, &fixture.slots[index], values.len())
                }
                Action::AppendFrame { index } | Action::AppendTask { index } => {
                    assert_eq!(index, fixture.slots.len());
                    if matches!(cursor.action(), Action::AppendFrame { .. }) {
                        let pending = cursor.action();
                        let record = cursor.record(index).unwrap();
                        assert_eq!(cursor.action(), pending); // refused append
                        let foreign_record =
                            CssColorSlot::new(&foreign, record.phase, index, record.data);
                        assert_eq!(
                            cursor.written(index, &foreign_record),
                            Err(Error::Protocol(ColorProtocolError::ForeignSlot))
                        );
                        assert_eq!(cursor.action(), pending);
                        let wrong = CssColorSlot::new(
                            record.owner,
                            record.phase,
                            index,
                            Data::NameUnit('x'),
                        );
                        assert_eq!(
                            cursor.written(index, &wrong),
                            Err(Error::Protocol(ColorProtocolError::WrongSlot))
                        );
                        assert_eq!(cursor.action(), pending);
                        let Data::Resolution(mut changed) = record.data else {
                            unreachable!()
                        };
                        changed.parent = Some(index);
                        let changed = CssColorSlot::new(
                            record.owner,
                            record.phase,
                            index,
                            Data::Resolution(changed),
                        );
                        assert_eq!(
                            cursor.written(index, &changed),
                            Err(Error::Protocol(ColorProtocolError::WrongSlot))
                        );
                        assert_eq!(cursor.action(), pending);
                        if cancel_at == Some(suspensions) {
                            cursor.abandon();
                            break Err(Error::Protocol(ColorProtocolError::Terminal));
                        }
                        suspensions += 1;
                    }
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
        assert_eq!(cursor.finish(), Err(ColorProtocolError::Terminal));
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
        let name = crate::color::CssColorProfileName::try_new("--printer").unwrap();
        let metadata = [CssColorProfileBinding::new(
            &name,
            profile,
            CssColorProfileAvailability::Loaded,
            crate::rule::color_profile::CssColorProfileRenderingIntent::RelativeColorimetric,
            3,
            &[],
        )];
        let input = [0.2, 0.3, 0.0];
        let expected = Color::new(
            Space::Predefined(Predefined::XyzD50),
            [
                surgeist_color::Finite::new(0.4),
                surgeist_color::Finite::new(0.5),
                surgeist_color::Finite::new(0.6),
            ],
            surgeist_color::UnitInterval::new(1.0),
        );
        let conversions = [CssColorProfileConversion::Resolved(
            CssColorProfileConversionRecord::new(
                profile,
                &input,
                crate::rule::color_profile::CssColorProfileRenderingIntent::RelativeColorimetric,
                Space::Predefined(Predefined::XyzD50),
                expected,
            ),
        )];
        let facts = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Light,
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
        let wrapped = resolve(
            "light-dark(color(--printer .2 .3 / .5), currentcolor)",
            &facts,
        )
        .unwrap();
        assert_eq!(wrapped.channels(), expected.channels());
        assert_eq!(wrapped.alpha(), Some(0.5));
        let relative =
            "color(from light-dark(color(--printer .2 .3 / .5), currentcolor) --printer .2 .3)";
        // The profile origin and selected wrapper each suspend before their
        // child and resume from the qualified parent slot afterward.
        for suspension in 0..4 {
            assert_eq!(
                resolve_suspending(relative, &facts, Some(suspension)),
                Err(Error::Protocol(ColorProtocolError::Terminal))
            );
        }
        let relative = resolve_suspending(relative, &facts, Some(4)).unwrap();
        assert_eq!(relative.channels(), expected.channels());
        assert_eq!(relative.alpha(), Some(0.5));
        let mixed = resolve(
            "color-mix(in xyz-d50, color(--printer .2 .3) 50%, color(xyz-d50 .4 .5 .6) 50%)",
            &facts,
        )
        .unwrap();
        assert_eq!(mixed.channels(), expected.channels());
        assert_eq!(mixed.alpha(), Some(1.0));
        assert_eq!(
            resolve(
                "light-dark(color(--printer .2 calc(.1 + .2)), currentcolor)",
                &facts
            ),
            Err(Error::Conversion(
                super::super::profile_conversion::Error::Unavailable
            ))
        );
    }
    #[test]
    fn nested_parent_suspensions_preserve_refusal_and_cancel_without_a_result() {
        let facts = facts(CssColorSchemeFact::Light, CssDeviceColorFact::Absent);
        let input = "color-mix(in srgb, rgb(from light-dark(red,currentcolor) calc(r / 2) g b) 50%, contrast-color(device-cmyk(0 0 0 1)) 50%)";
        // Five actual child descents: two mix members, the relative origin,
        // the selected light arm and contrast's device input. Each can stop
        // before storing its parent and again before reading that parent back.
        for suspension in 0..10 {
            assert_eq!(
                resolve_suspending(input, &facts, Some(suspension)),
                Err(Error::Protocol(ColorProtocolError::Terminal))
            );
        }
        let recovered = resolve_suspending(input, &facts, Some(10)).unwrap();
        assert_eq!(recovered.channels(), [Some(0.75), Some(0.5), Some(0.5)]);
        assert_eq!(recovered.alpha(), Some(1.0));
    }
    #[test]
    fn exclusive_frame_families_resume_nested_mix_wrapper_and_coordinate_children() {
        let facts = facts(CssColorSchemeFact::Light, CssDeviceColorFact::Absent);
        let text = "color-mix(in srgb, rgb(from light-dark(red,currentcolor) calc(r / 2) g b) 50%, contrast-color(device-cmyk(0 0 0 1)) 50%)";
        let value = resolve(text, &facts).unwrap();
        assert_eq!(value.channels(), [Some(0.75), Some(0.5), Some(0.5)]);
        assert_eq!(value.alpha(), Some(1.0));
        assert!(
            (value.serialization_family() == surgeist_color::SerializationFamily::ColorFunction)
        );
    }
}
