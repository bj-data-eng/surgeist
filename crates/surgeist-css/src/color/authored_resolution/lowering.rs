//! Complete typed Color DFS followed by contiguous canonical component runs.

use super::super::{
    component::{CheckedScalar, ComponentContext, SyntaxFailure},
    grammar::{
        ComponentRecord, ComponentRole, FunctionRecipe, GrammarInput, GrammarRecord, GrammarTable,
        Interpolation, Leaf, MixRecord, ProfileSpace, RawComponent, Recipe, RecipeKind, Site,
    },
    keywords::ColorFunction as F,
    protocol::{
        ColorProtocolError, CssColorBank, CssColorOperationOwner, CssColorSlot, Data, RecordRange,
        RecordTable,
    },
    syntax_pass::ValidatedGrammar,
};
use super::position::Position;
use super::{
    CssColorPathStep,
    input::{self, Fields, Operand, Shape},
    scalar::{self, ScalarLowering},
    site::SiteRecord,
    spelling::{self, NameSpelling},
};
use crate::CssColor;
use crate::syntax::color::operands::{self, ColorHeader, ColorNode};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Stage {
    Begin,
    Source,
    First,
    Second,
    MixColor,
    MixWeight,
    MixCommit,
    Channels,
    Alpha,
    Flush,
    FlushAlpha,
    Finish,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct PendingComponent {
    site: usize,
    checked: CheckedScalar,
    context: ComponentContext,
    role: ComponentRole,
    ordinal: usize,
}
impl PendingComponent {
    fn record(self, previous: Option<usize>) -> ComponentRecord {
        ComponentRecord {
            previous,
            ordinal: self.ordinal,
            role: self.role,
            component: RawComponent::Authored { site: self.site },
            context: self.context,
            checked: Some(self.checked),
        }
    }
}
#[derive(Clone, Copy)]
struct FrameInput<'css> {
    alpha: bool,
    source: bool,
    children: [bool; 2],
    profile: Option<&'css crate::CssColorProfileName>,
    interpolation: Option<&'css crate::CssColorInterpolation>,
}
#[derive(Clone, Copy)]
pub(crate) struct Frame<'css> {
    color: &'css CssColor,
    node: ColorNode<'css>,
    input: FrameInput<'css>,
    recipe: FunctionRecipe,
    parent: Option<usize>,
    site: usize,
    depth: usize,
    closed: bool,
    stage: Stage,
    ordinal: usize,
    fixed: [Option<PendingComponent>; 4],
    alpha: Option<PendingComponent>,
    stage_head: Option<usize>,
    stage_tail: Option<usize>,
    flush_next: Option<usize>,
    mix_color: Option<usize>,
    mix_weight: Option<usize>,
    expected: Option<Operand<'css>>,
}
impl std::fmt::Debug for Frame<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthoredColorFrame")
            .field("color", &std::ptr::from_ref(self.color))
            .field("site", &self.site)
            .field("stage", &self.stage)
            .field("ordinal", &self.ordinal)
            .finish_non_exhaustive()
    }
}
impl Frame<'_> {
    fn identical(self, other: Self) -> bool {
        std::ptr::eq(self.color, other.color)
            && self.recipe == other.recipe
            && self.parent == other.parent
            && self.site == other.site
            && self.depth == other.depth
            && self.closed == other.closed
            && self.stage == other.stage
            && self.ordinal == other.ordinal
            && self.fixed == other.fixed
            && self.alpha == other.alpha
            && self.stage_head == other.stage_head
            && self.stage_tail == other.stage_tail
            && self.flush_next == other.flush_next
            && self.mix_color == other.mix_color
            && self.mix_weight == other.mix_weight
            && match (self.expected, other.expected) {
                (Some(a), Some(b)) => input::same_operand(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct StagedComponent {
    payload: PendingComponent,
    next: Option<usize>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Action {
    ReadColor,
    ReadPayload { bytes: usize },
    ReadChild { edge: CssColorPathStep },
    ReadProfileHeader,
    ReadHexByte { offset: usize },
    ReadByte { offset: usize },
    Scalar(scalar::Action),
    Read { bank: CssColorBank, index: usize },
    Append { bank: CssColorBank, index: usize },
    Replace { bank: CssColorBank, index: usize },
    Compute,
    Done,
    Failed(SyntaxFailure),
}
#[derive(Clone, Copy)]
struct NewColor<'css> {
    color: &'css CssColor,
    parent: Option<usize>,
    parent_site: Option<usize>,
    depth: usize,
    edge: Option<CssColorPathStep>,
}
#[derive(Clone, Copy)]
enum Commit<'css> {
    Suspend {
        frame: Frame<'css>,
        index: usize,
        child: &'css CssColor,
        edge: CssColorPathStep,
    },
    Recipe {
        color: &'css CssColor,
        parent: Option<usize>,
        root: usize,
    },
    Staged {
        frame: Frame<'css>,
        index: usize,
    },
    Linked(Frame<'css>),
    Component(Frame<'css>),
    Alpha(Frame<'css>),
    Weight(Frame<'css>),
    Mix(Frame<'css>),
}
#[expect(
    clippy::large_enum_variant,
    reason = "State variants retain the current parent frame and at most one active scalar child inline; the caller admits this fixed layout without hidden heap frames"
)]
enum State<'css> {
    Color(NewColor<'css>),
    Site {
        input: NewColor<'css>,
        header: ColorHeader<'css>,
    },
    Payload {
        input: NewColor<'css>,
        header: ColorHeader<'css>,
        site: usize,
    },
    Hex {
        input: NewColor<'css>,
        site: usize,
        digits: &'css str,
        offset: usize,
        bytes: [u8; 4],
    },
    Frame(Frame<'css>),
    Child {
        frame: Frame<'css>,
        edge: CssColorPathStep,
    },
    ProfileHeader(Frame<'css>),
    InterpolationProfile {
        frame: Frame<'css>,
        profile: &'css crate::CssColorProfileName,
    },
    Name {
        frame: Frame<'css>,
        writer: NameSpelling<'css>,
        interpolation: bool,
    },
    Scalar {
        frame: Frame<'css>,
        cursor: ScalarLowering<'css>,
    },
    Parent {
        index: usize,
        color: &'css CssColor,
        root: usize,
    },
    StageTail {
        frame: Frame<'css>,
        new: usize,
    },
    Flush {
        frame: Frame<'css>,
        index: usize,
    },
    Pending {
        data: Data<'css>,
        replace: Option<usize>,
        commit: Commit<'css>,
    },
    Done {
        root: usize,
    },
}
pub(super) struct Lowering<'css> {
    owner: &'css CssColorOperationOwner,
    phase: usize,
    state: State<'css>,
    algorithm_end: usize,
    arithmetic_end: usize,
    position: Position,
}
impl<'css> Lowering<'css> {
    pub(super) fn start(
        owner: &'css CssColorOperationOwner,
        phase: usize,
        color: &'css CssColor,
    ) -> Self {
        Self {
            owner,
            phase,
            state: State::Color(NewColor {
                color,
                parent: None,
                parent_site: None,
                depth: 0,
                edge: None,
            }),
            algorithm_end: 0,
            arithmetic_end: 0,
            position: Position {
                site: 0,
                depth: 0,
                edge: None,
            },
        }
    }
    pub(super) fn action(&self) -> Action {
        match &self.state {
            State::Color(_) => Action::ReadColor,
            State::Site { .. } => Action::Append {
                bank: CssColorBank::Algorithm,
                index: self.algorithm_end,
            },
            State::Payload { header, .. } => Action::ReadPayload {
                bytes: header.node.local_bytes(),
            },
            State::Hex { digits, offset, .. } if *offset < digits.len() => {
                Action::ReadHexByte { offset: *offset }
            }
            State::Hex { .. } | State::Frame(_) => Action::Compute,
            State::Child { edge, .. } => Action::ReadChild { edge: *edge },
            State::ProfileHeader(_) | State::InterpolationProfile { .. } => {
                Action::ReadProfileHeader
            }
            State::Name { writer, .. } => match writer.action() {
                spelling::Action::ReadByte { offset } => Action::ReadByte { offset },
                spelling::Action::Append { index } => Action::Append {
                    bank: CssColorBank::Algorithm,
                    index,
                },
                _ => Action::Compute,
            },
            State::Scalar { cursor, .. } => match cursor.action() {
                scalar::Action::Read { bank, index } => Action::Read { bank, index },
                scalar::Action::Append { bank, index } => Action::Append { bank, index },
                scalar::Action::Replace { bank, index } => Action::Replace { bank, index },
                scalar::Action::Compute | scalar::Action::Done => Action::Compute,
                scalar::Action::Failed(f) => Action::Failed(f),
                action => Action::Scalar(action),
            },
            State::Parent { index, .. } | State::Flush { index, .. } => Action::Read {
                bank: CssColorBank::Algorithm,
                index: *index,
            },
            State::StageTail { frame, .. } => Action::Read {
                bank: CssColorBank::Algorithm,
                index: frame.stage_tail.expect("tail exists before linking"),
            },
            State::Pending { replace, .. } => match replace {
                Some(index) => Action::Replace {
                    bank: CssColorBank::Algorithm,
                    index: *index,
                },
                None => Action::Append {
                    bank: CssColorBank::Algorithm,
                    index: self.algorithm_end,
                },
            },
            State::Done { .. } => Action::Done,
        }
    }
    pub(super) fn input_bytes(&self) -> Result<usize, ColorProtocolError> {
        Ok(match &self.state {
            State::Color(_) => operands::header_bytes(),
            State::Payload { header, .. } => header.node.local_bytes(),
            State::Child { frame, edge } => input::child_bytes(frame.node, *edge)?,
            State::InterpolationProfile { .. } => std::mem::size_of::<crate::CssColorProfileName>(),
            State::ProfileHeader(frame) => {
                if frame.input.profile.is_some() {
                    std::mem::size_of::<crate::CssColorProfileName>()
                } else {
                    std::mem::size_of::<crate::CssColorInterpolation>()
                }
            }
            State::Hex { digits, offset, .. } if *offset < digits.len() => 1,
            State::Name { .. } if matches!(self.action(), Action::ReadByte { .. }) => 1,
            State::Scalar { cursor, .. } => cursor.input_bytes()?,
            _ => 0,
        })
    }
    fn pending(
        &mut self,
        data: Data<'css>,
        replace: Option<usize>,
        commit: Commit<'css>,
    ) -> Result<(), ColorProtocolError> {
        if replace.is_none() {
            self.algorithm_end
                .checked_add(1)
                .ok_or(ColorProtocolError::Overflow)?;
        }
        self.state = State::Pending {
            data,
            replace,
            commit,
        };
        Ok(())
    }
    fn recipe(
        &mut self,
        input: NewColor<'css>,
        site: usize,
        kind: RecipeKind,
        closed: bool,
    ) -> Result<(), ColorProtocolError> {
        self.pending(
            Data::Grammar(GrammarRecord::Recipe(Recipe {
                kind,
                site: Site::Authored { record: site },
                closed,
            })),
            None,
            Commit::Recipe {
                color: input.color,
                parent: input.parent,
                root: self.algorithm_end,
            },
        )
    }
    pub(super) fn read_authored(&mut self) -> Result<(), ColorProtocolError> {
        match &mut self.state {
            State::Color(input) => {
                let input = *input;
                let header = operands::header(input.color);
                self.state = State::Site { input, header };
                Ok(())
            }
            State::Payload {
                input,
                header,
                site,
            } => {
                let input = *input;
                let header = *header;
                let site = *site;
                match input::shape(header.node) {
                    Shape::Leaf(leaf) => {
                        self.recipe(input, site, RecipeKind::Leaf(leaf), header.closed)
                    }
                    Shape::Hex(digits) => {
                        self.state = State::Hex {
                            input,
                            site,
                            digits,
                            offset: 0,
                            bytes: [0; 4],
                        };
                        Ok(())
                    }
                    Shape::Function(fields) => {
                        let Fields {
                            function,
                            space,
                            syntax,
                            count,
                            alpha,
                            source,
                            children,
                            mix_count,
                            profile,
                            interpolation,
                        } = fields;
                        let recipe = FunctionRecipe {
                            function,
                            origin: None,
                            space: space.map(ProfileSpace::Predefined),
                            syntax,
                            components: None,
                            component_start: None,
                            count,
                            alpha: None,
                            children: [None; 2],
                            mix: None,
                            mix_count,
                            interpolation: None,
                        };
                        let plan = FrameInput {
                            alpha,
                            source,
                            children,
                            profile,
                            interpolation,
                        };
                        self.state = State::Frame(Frame {
                            color: input.color,
                            node: header.node,
                            input: plan,
                            recipe,
                            parent: input.parent,
                            site,
                            depth: input.depth,
                            closed: header.closed,
                            stage: Stage::Begin,
                            ordinal: 0,
                            fixed: [None; 4],
                            alpha: None,
                            stage_head: None,
                            stage_tail: None,
                            flush_next: None,
                            mix_color: None,
                            mix_weight: None,
                            expected: None,
                        });
                        Ok(())
                    }
                }
            }
            State::Hex {
                digits,
                offset,
                bytes,
                ..
            } => {
                let byte = *digits
                    .as_bytes()
                    .get(*offset)
                    .ok_or(ColorProtocolError::WrongIndex)?;
                let next = offset.checked_add(1).ok_or(ColorProtocolError::Overflow)?;
                if !super::super::hex::write_digit(bytes, digits.len(), *offset, char::from(byte)) {
                    return Err(ColorProtocolError::Input);
                }
                *offset = next;
                Ok(())
            }
            State::Child { frame, edge } => {
                let mut frame = *frame;
                let edge = *edge;
                let operand = input::child(frame.node, edge);
                let Some(operand) = operand else {
                    if matches!(edge, CssColorPathStep::MixWeight(_)) {
                        frame.stage = Stage::MixCommit;
                        self.state = State::Frame(frame);
                        return Ok(());
                    }
                    return Err(ColorProtocolError::WrongSlot);
                };
                frame.expected = Some(operand);
                if let Operand::Color(child) = operand {
                    self.pending(
                        Data::AuthoredColorFrame(frame),
                        None,
                        Commit::Suspend {
                            frame,
                            index: self.algorithm_end,
                            child,
                            edge,
                        },
                    )
                } else {
                    let role = role(frame.recipe, edge, operand);
                    let ordinal = frame.ordinal;
                    let context = ComponentContext::for_recipe(frame.recipe, role, ordinal)?;
                    let depth = frame
                        .depth
                        .checked_add(1)
                        .ok_or(ColorProtocolError::Overflow)?;
                    self.state = State::Scalar {
                        frame,
                        cursor: ScalarLowering::start(
                            self.owner,
                            self.phase,
                            scalar::ScalarInput {
                                operand,
                                parent_site: frame.site,
                                depth,
                                edge,
                                context,
                            },
                            self.algorithm_end,
                            self.arithmetic_end,
                        ),
                    };
                    Ok(())
                }
            }
            State::ProfileHeader(frame) => {
                let frame = *frame;
                if let Some(profile) = frame.input.profile {
                    self.state = State::Name {
                        frame,
                        writer: NameSpelling::start(
                            self.owner,
                            self.phase,
                            profile.as_str(),
                            frame.site,
                            self.algorithm_end,
                        ),
                        interpolation: false,
                    };
                } else if let Some(interpolation) = frame.input.interpolation {
                    if let Some(method) = interpolation.predefined() {
                        let mut frame = frame;
                        frame.recipe.interpolation = Some(Interpolation::Standard(method));
                        frame.stage = Stage::Source;
                        self.state = State::Frame(frame);
                    } else if let Some(profile) = interpolation.custom_profile() {
                        self.state = State::InterpolationProfile { frame, profile };
                    } else {
                        return Err(ColorProtocolError::WrongSlot);
                    }
                } else {
                    return Err(ColorProtocolError::WrongSlot);
                }
                Ok(())
            }
            State::InterpolationProfile { frame, profile } => {
                let frame = *frame;
                self.state = State::Name {
                    frame,
                    writer: NameSpelling::start(
                        self.owner,
                        self.phase,
                        profile.as_str(),
                        frame.site,
                        self.algorithm_end,
                    ),
                    interpolation: true,
                };
                Ok(())
            }
            State::Name { writer, .. } => writer.read_byte(),
            State::Scalar { cursor, .. } => cursor.read_authored(),
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
            State::Scalar { cursor, .. } => cursor.read(bank, index, slot),
            State::Parent {
                index: expected,
                color,
                root,
            } => {
                if index != *expected {
                    return Err(ColorProtocolError::WrongIndex);
                }
                let Data::AuthoredColorFrame(mut frame) = slot.data else {
                    return Err(ColorProtocolError::WrongSlot);
                };
                if !matches!(frame.expected,Some(Operand::Color(expected)) if std::ptr::eq(expected,*color))
                {
                    return Err(ColorProtocolError::WrongSlot);
                }
                let root = *root;
                match frame.stage {
                    Stage::Source => {
                        if frame.recipe.function == F::Contrast {
                            frame.recipe.children[0] = Some(root);
                        } else {
                            frame.recipe.origin = Some(root);
                        }
                        frame.stage = Stage::First;
                    }
                    Stage::First => {
                        frame.recipe.children[0] = Some(root);
                        frame.stage = Stage::Second;
                    }
                    Stage::Second => {
                        frame.recipe.children[1] = Some(root);
                        frame.stage = Stage::MixColor;
                    }
                    Stage::MixColor => {
                        frame.mix_color = Some(root);
                        frame.stage = Stage::MixWeight;
                    }
                    _ => return Err(ColorProtocolError::WrongSlot),
                }
                frame.expected = None;
                self.position = Position {
                    site: frame.site,
                    depth: frame.depth,
                    edge: None,
                };
                self.state = State::Frame(frame);
                Ok(())
            }
            State::StageTail { frame, new, .. } => {
                let frame = *frame;
                let new = *new;
                let Data::AuthoredStagedComponent(mut staged) = slot.data else {
                    return Err(ColorProtocolError::WrongSlot);
                };
                if staged.next.is_some() {
                    return Err(ColorProtocolError::WrongSlot);
                }
                staged.next = Some(new);
                self.pending(
                    Data::AuthoredStagedComponent(staged),
                    Some(index),
                    Commit::Linked(frame),
                )
            }
            State::Flush { frame, .. } => {
                let mut frame = *frame;
                let Data::AuthoredStagedComponent(staged) = slot.data else {
                    return Err(ColorProtocolError::WrongSlot);
                };
                if staged.payload.ordinal != frame.ordinal {
                    return Err(ColorProtocolError::WrongSlot);
                }
                frame.flush_next = staged.next;
                self.pending(
                    Data::Grammar(GrammarRecord::Component(
                        staged.payload.record(frame.recipe.components),
                    )),
                    None,
                    Commit::Component(frame),
                )
            }
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    fn pending_site(&self) -> Result<SiteRecord<'css>, ColorProtocolError> {
        let State::Site { input, .. } = self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        Ok(SiteRecord {
            operand: Operand::Color(input.color),
            parent: input.parent_site,
            edge: input.edge,
            depth: input.depth,
            origin: None,
        })
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
        let data = match &self.state {
            State::Site { .. } => Data::AuthoredSite(self.pending_site()?),
            State::Pending { data, .. } => *data,
            State::Name { writer, .. } => return writer.record(index),
            State::Scalar { cursor, .. } => return cursor.record(bank, index),
            _ => return Err(ColorProtocolError::WrongAction),
        };
        Ok(CssColorSlot::new(self.owner, self.phase, index, data))
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
            State::Name { writer, .. } => {
                writer.written(index, slot)?;
                self.algorithm_end = writer.algorithm_end();
                return Ok(());
            }
            State::Scalar { cursor, .. } => {
                cursor.written(bank, index, slot)?;
                self.algorithm_end = cursor.algorithm_end();
                self.arithmetic_end = cursor.arithmetic_end();
                return Ok(());
            }
            _ => {}
        }
        if let State::Site { input, header } = self.state {
            let Data::AuthoredSite(actual) = slot.data else {
                return Err(ColorProtocolError::WrongSlot);
            };
            if !self.pending_site()?.identical(actual) {
                return Err(ColorProtocolError::WrongSlot);
            }
            self.algorithm_end = self
                .algorithm_end
                .checked_add(1)
                .ok_or(ColorProtocolError::Overflow)?;
            self.state = State::Payload {
                input,
                header,
                site: index,
            };
            return Ok(());
        }
        let State::Pending {
            data,
            replace,
            commit,
        } = self.state
        else {
            return Err(ColorProtocolError::WrongAction);
        };
        if !same_data(data, slot.data) {
            return Err(ColorProtocolError::WrongSlot);
        }
        if replace.is_none() {
            self.algorithm_end = self
                .algorithm_end
                .checked_add(1)
                .ok_or(ColorProtocolError::Overflow)?;
        }
        match commit {
            Commit::Suspend {
                frame,
                index,
                child,
                edge,
            } => {
                let depth = frame
                    .depth
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                self.position = Position {
                    site: self.algorithm_end,
                    depth,
                    edge: Some(edge),
                };
                self.state = State::Color(NewColor {
                    color: child,
                    parent: Some(index),
                    parent_site: Some(frame.site),
                    depth,
                    edge: Some(edge),
                });
            }
            Commit::Recipe {
                color,
                parent,
                root,
            } => {
                self.state = match parent {
                    Some(index) => State::Parent { index, color, root },
                    None => State::Done { root },
                }
            }
            Commit::Staged { mut frame, index } => {
                if frame.stage_tail.is_some() {
                    self.state = State::StageTail { frame, new: index };
                } else {
                    frame.stage_head = Some(index);
                    frame.stage_tail = Some(index);
                    frame.ordinal += 1;
                    self.state = State::Frame(frame);
                }
            }
            Commit::Linked(mut frame) => {
                let State::Pending {
                    data: Data::AuthoredStagedComponent(staged),
                    ..
                } = self.state
                else {
                    unreachable!("linked staging record")
                };
                frame.stage_tail = staged.next;
                frame.ordinal = frame
                    .ordinal
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                self.state = State::Frame(frame);
            }
            Commit::Component(mut frame) => {
                frame.recipe.component_start.get_or_insert(index);
                frame.recipe.components = Some(index);
                frame.ordinal = frame
                    .ordinal
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                self.state = State::Frame(frame);
            }
            Commit::Alpha(mut frame) => {
                frame.recipe.alpha = Some(index);
                frame.stage = Stage::Finish;
                self.state = State::Frame(frame);
            }
            Commit::Weight(mut frame) => {
                frame.mix_weight = Some(index);
                frame.stage = Stage::MixCommit;
                self.state = State::Frame(frame);
            }
            Commit::Mix(mut frame) => {
                frame.recipe.mix = Some(index);
                frame.ordinal = frame
                    .ordinal
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                frame.mix_color = None;
                frame.mix_weight = None;
                frame.stage = Stage::MixColor;
                self.state = State::Frame(frame);
            }
        }
        Ok(())
    }
    pub(super) fn kernel(&self) -> crate::color::CssColorKernel {
        match &self.state {
            State::Scalar { cursor, .. } => cursor.kernel(),
            _ => crate::color::CssColorKernel::Transition,
        }
    }
    pub(super) fn compute(&mut self) -> Result<(), ColorProtocolError> {
        if self.action() != Action::Compute {
            return Err(ColorProtocolError::WrongAction);
        }
        match &mut self.state {
            State::Hex {
                input,
                site,
                digits,
                bytes,
                ..
            } => {
                let input = *input;
                let site = *site;
                let value = super::super::hex::finish(*bytes, digits.len());
                self.recipe(input, site, RecipeKind::Leaf(Leaf::Hex(value)), true)
            }
            State::Name {
                frame,
                writer,
                interpolation,
            } => {
                if let spelling::Action::Done { record } = writer.action() {
                    let mut frame = *frame;
                    let units = RecordRange {
                        start: writer.units_start(),
                        end: record,
                    };
                    if *interpolation {
                        frame.recipe.interpolation = Some(Interpolation::Custom {
                            token: record,
                            units,
                        });
                    } else {
                        frame.recipe.space = Some(ProfileSpace::Custom {
                            token: record,
                            units,
                        });
                    }
                    frame.stage = Stage::Source;
                    self.state = State::Frame(frame);
                    Ok(())
                } else {
                    writer.compute()
                }
            }
            State::Scalar { frame, cursor } => {
                if cursor.action() != scalar::Action::Done {
                    return cursor.compute();
                }
                let mut frame = *frame;
                let (site, checked) = cursor.finish()?;
                self.position = Position {
                    site: frame.site,
                    depth: frame.depth,
                    edge: None,
                };
                let operand = frame.expected.ok_or(ColorProtocolError::WrongSlot)?;
                let edge = match frame.stage {
                    Stage::Channels => CssColorPathStep::Channel(frame.ordinal),
                    Stage::Alpha => CssColorPathStep::Alpha,
                    Stage::MixWeight => CssColorPathStep::MixWeight(frame.ordinal),
                    _ => return Err(ColorProtocolError::WrongSlot),
                };
                let role = role(frame.recipe, edge, operand);
                let context = ComponentContext::for_recipe(frame.recipe, role, frame.ordinal)?;
                let payload = PendingComponent {
                    site,
                    checked,
                    context,
                    role,
                    ordinal: frame.ordinal,
                };
                frame.expected = None;
                match frame.stage {
                    Stage::Alpha => {
                        frame.alpha = Some(payload);
                        frame.stage = Stage::Flush;
                        frame.ordinal = 0;
                        frame.flush_next = frame.stage_head;
                        self.state = State::Frame(frame);
                        Ok(())
                    }
                    Stage::MixWeight => self.pending(
                        Data::Grammar(GrammarRecord::Component(payload.record(None))),
                        None,
                        Commit::Weight(frame),
                    ),
                    Stage::Channels if frame.input.profile.is_some() => self.pending(
                        Data::AuthoredStagedComponent(StagedComponent {
                            payload,
                            next: None,
                        }),
                        None,
                        Commit::Staged {
                            frame,
                            index: self.algorithm_end,
                        },
                    ),
                    Stage::Channels => {
                        let slot = frame
                            .fixed
                            .get_mut(frame.ordinal)
                            .ok_or(ColorProtocolError::WrongIndex)?;
                        *slot = Some(payload);
                        frame.ordinal = frame
                            .ordinal
                            .checked_add(1)
                            .ok_or(ColorProtocolError::Overflow)?;
                        self.state = State::Frame(frame);
                        Ok(())
                    }
                    _ => Err(ColorProtocolError::WrongSlot),
                }
            }
            State::Frame(frame) => {
                let mut frame = *frame;
                match frame.stage {
                    Stage::Begin => {
                        if frame.input.profile.is_some() || frame.input.interpolation.is_some() {
                            self.state = State::ProfileHeader(frame);
                        } else {
                            frame.stage = Stage::Source;
                            self.state = State::Frame(frame);
                        }
                        Ok(())
                    }
                    Stage::Source => {
                        if frame.input.source {
                            self.state = State::Child {
                                frame,
                                edge: CssColorPathStep::Source,
                            };
                        } else {
                            frame.stage = Stage::First;
                            self.state = State::Frame(frame);
                        }
                        Ok(())
                    }
                    Stage::First => {
                        if frame.input.children[0] {
                            self.state = State::Child {
                                frame,
                                edge: CssColorPathStep::First,
                            };
                        } else {
                            frame.stage = Stage::Second;
                            self.state = State::Frame(frame);
                        }
                        Ok(())
                    }
                    Stage::Second => {
                        if frame.input.children[1] {
                            self.state = State::Child {
                                frame,
                                edge: CssColorPathStep::Second,
                            };
                        } else {
                            frame.stage = Stage::MixColor;
                            frame.ordinal = 0;
                            self.state = State::Frame(frame);
                        }
                        Ok(())
                    }
                    Stage::MixColor if frame.ordinal < frame.recipe.mix_count => {
                        self.state = State::Child {
                            frame,
                            edge: CssColorPathStep::MixColor(frame.ordinal),
                        };
                        Ok(())
                    }
                    Stage::MixColor => {
                        frame.stage = Stage::Channels;
                        frame.ordinal = 0;
                        self.state = State::Frame(frame);
                        Ok(())
                    }
                    Stage::MixWeight => {
                        self.state = State::Child {
                            frame,
                            edge: CssColorPathStep::MixWeight(frame.ordinal),
                        };
                        Ok(())
                    }
                    Stage::MixCommit => self.pending(
                        Data::Grammar(GrammarRecord::Mix(MixRecord {
                            previous: frame.recipe.mix,
                            ordinal: frame.ordinal,
                            color: frame.mix_color.ok_or(ColorProtocolError::WrongSlot)?,
                            weight: frame.mix_weight,
                        })),
                        None,
                        Commit::Mix(frame),
                    ),
                    Stage::Channels if frame.ordinal < frame.recipe.count => {
                        self.state = State::Child {
                            frame,
                            edge: CssColorPathStep::Channel(frame.ordinal),
                        };
                        Ok(())
                    }
                    Stage::Channels => {
                        frame.stage = Stage::Alpha;
                        self.state = State::Frame(frame);
                        Ok(())
                    }
                    Stage::Alpha => {
                        if frame.input.alpha {
                            self.state = State::Child {
                                frame,
                                edge: CssColorPathStep::Alpha,
                            };
                        } else {
                            frame.stage = Stage::Flush;
                            frame.ordinal = 0;
                            frame.flush_next = frame.stage_head;
                            self.state = State::Frame(frame);
                        }
                        Ok(())
                    }
                    Stage::Flush if frame.ordinal < frame.recipe.count => {
                        if frame.input.profile.is_some() {
                            self.state = State::Flush {
                                frame,
                                index: frame.flush_next.ok_or(ColorProtocolError::WrongSlot)?,
                            };
                            Ok(())
                        } else {
                            let payload =
                                frame.fixed[frame.ordinal].ok_or(ColorProtocolError::WrongSlot)?;
                            self.pending(
                                Data::Grammar(GrammarRecord::Component(
                                    payload.record(frame.recipe.components),
                                )),
                                None,
                                Commit::Component(frame),
                            )
                        }
                    }
                    Stage::Flush => {
                        frame.stage = Stage::FlushAlpha;
                        self.state = State::Frame(frame);
                        Ok(())
                    }
                    Stage::FlushAlpha => {
                        if let Some(alpha) = frame.alpha {
                            self.pending(
                                Data::Grammar(GrammarRecord::Component(alpha.record(None))),
                                None,
                                Commit::Alpha(frame),
                            )
                        } else {
                            frame.stage = Stage::Finish;
                            self.state = State::Frame(frame);
                            Ok(())
                        }
                    }
                    Stage::Finish => self.recipe(
                        NewColor {
                            color: frame.color,
                            parent: frame.parent,
                            parent_site: None,
                            depth: frame.depth,
                            edge: None,
                        },
                        frame.site,
                        RecipeKind::Function(frame.recipe),
                        frame.closed,
                    ),
                }
            }
            _ => Err(ColorProtocolError::WrongAction),
        }
    }
    pub(super) fn finish(&self) -> Result<ValidatedGrammar<'css>, ColorProtocolError> {
        let State::Done { root } = self.state else {
            return Err(ColorProtocolError::Terminal);
        };
        Ok(ValidatedGrammar {
            table: GrammarTable {
                input: GrammarInput::Authored(RecordTable {
                    owner: self.owner,
                    phase: self.phase,
                    length: self.algorithm_end,
                }),
                start: 0,
                end: self.algorithm_end,
                root,
            },
            expression_end: self.arithmetic_end,
        })
    }
    pub(super) fn position(&self) -> Position {
        match &self.state {
            State::Scalar { cursor, .. } => cursor.position(),
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
fn role(recipe: FunctionRecipe, edge: CssColorPathStep, operand: Operand<'_>) -> ComponentRole {
    match edge {
        CssColorPathStep::Alpha => ComponentRole::Alpha,
        CssColorPathStep::MixWeight(_) => ComponentRole::Weight,
        _ if matches!(recipe.space, Some(ProfileSpace::Custom { .. })) => {
            ComponentRole::ProfileChannel
        }
        _ if recipe.origin.is_some() => ComponentRole::RelativeChannel,
        _ if matches!(operand, Operand::Hue(_)) => ComponentRole::Hue,
        _ => ComponentRole::Channel,
    }
}
fn same_data(left: Data<'_>, right: Data<'_>) -> bool {
    match (left, right) {
        (Data::AuthoredColorFrame(a), Data::AuthoredColorFrame(b)) => a.identical(b),
        (Data::AuthoredStagedComponent(a), Data::AuthoredStagedComponent(b)) => a == b,
        (Data::Grammar(a), Data::Grammar(b)) => a == b,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{
        facts::{
            CssColorFactIdentity, CssColorFacts, CssColorPropertyFacts, CssColorSchemeFact,
            CssDeviceColorFact,
        },
        operation as core,
    };
    use surgeist_color::{Color, ColorSpace, PredefinedColorSpace};

    /// The `<color>` the production `color` property parser admits for `source`.
    fn parsed(source: &str) -> CssColor {
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
    fn resolve(root: &CssColor, facts: &CssColorFacts<'_>) -> Result<Color, core::Error> {
        let owner = CssColorOperationOwner::new();
        let phase = owner.current_phase();
        let next = owner.prepare_advance(phase).unwrap();
        owner.commit_phase(phase, next);
        let mut banks: [Vec<CssColorSlot<'_>>; 3] = std::array::from_fn(|_| Vec::new());
        let mut lower = Lowering::start(&owner, next, root);
        let mut done = false;
        for _ in 0..30_000_000 {
            match lower.action() {
                Action::ReadColor
                | Action::ReadPayload { .. }
                | Action::ReadChild { .. }
                | Action::ReadProfileHeader
                | Action::ReadHexByte { .. }
                | Action::ReadByte { .. }
                | Action::Scalar(_) => {
                    let _ = lower.input_bytes().unwrap();
                    lower.read_authored().unwrap();
                }
                Action::Read { bank, index } => lower
                    .read(bank, index, &banks[ordinal(bank)][index])
                    .unwrap(),
                Action::Append { bank, index } | Action::Replace { bank, index } => {
                    let replacing = matches!(lower.action(), Action::Replace { .. });
                    let slots = &mut banks[ordinal(bank)];
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
                Action::Failed(failure) => panic!("unexpected checked syntax failure: {failure:?}"),
            }
        }
        assert!(done, "lowering did not finish");
        assert_eq!(lower.algorithm_end(), banks[0].len());
        assert_eq!(lower.arithmetic_end(), banks[1].len());
        // Every function's immutable component run is actually contiguous.
        for slot in &banks[0] {
            if let Data::Grammar(GrammarRecord::Recipe(Recipe {
                kind: RecipeKind::Function(recipe),
                ..
            })) = slot.data
            {
                for index in 0..recipe.count {
                    let at = recipe.component_index(index).unwrap();
                    let Data::Grammar(GrammarRecord::Component(component)) = banks[0][at].data
                    else {
                        panic!("interrupted component run")
                    };
                    assert_eq!(component.ordinal, index);
                }
            }
        }
        let lengths = [banks[0].len(), banks[1].len(), banks[2].len()];
        let mut cursor = core::ColorOperation::start_authored(
            &lower.finish().unwrap(),
            facts,
            facts.identity(),
            lengths,
        )
        .unwrap();
        for _ in 0..30_000_000 {
            if cursor.action() == core::Action::UsedReady {
                return Ok(cursor.take_authored().unwrap().unwrap());
            }
            if cursor.action() == core::Action::StandardFailed {
                panic!(
                    "unexpected standard failure {:?}",
                    cursor.take_authored().unwrap().unwrap_err()
                );
            }
            match cursor.action().storage() {
                Some(core::StorageAction::Read { bank, index }) => {
                    cursor.read_slot(bank, index, &banks[ordinal(bank)][index])?
                }
                Some(core::StorageAction::Append { bank, index })
                | Some(core::StorageAction::Replace { bank, index }) => {
                    let replacing = matches!(
                        cursor.action().storage(),
                        Some(core::StorageAction::Replace { .. })
                    );
                    let slots = &mut banks[ordinal(bank)];
                    let record = cursor.pending_record(bank, index)?;
                    if replacing {
                        slots[index] = record;
                    } else {
                        assert_eq!(index, slots.len());
                        slots.push(record);
                    }
                    cursor.written(bank, index, &slots[index])?;
                }
                None => cursor.compute()?,
            }
        }
        panic!("resolution did not finish")
    }
    fn ordinal(bank: CssColorBank) -> usize {
        match bank {
            CssColorBank::Algorithm => 0,
            CssColorBank::Arithmetic => 1,
            CssColorBank::Evaluation => 2,
        }
    }
    #[test]
    fn authored_color_recipes_use_literal_results_and_original_child_order() {
        let id = CssColorFactIdentity::new(2, 3);
        let palette = [crate::CssColorPaletteEntry::new(
            crate::CssSystemColor::CanvasText,
            id,
            Color::from_srgba8([255, 0, 0, 255]),
        )];
        let facts = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Current(Color::from_srgba8([0, 255, 0, 255])),
            CssColorSchemeFact::Light,
            &palette,
            &[],
            &[],
            CssDeviceColorFact::Absent,
        );
        let rgb = ColorSpace::Predefined(PredefinedColorSpace::Srgb);
        for (source, space, channels, alpha) in [
            ("currentcolor", rgb, [0.0, 1.0, 0.0], 1.0),
            ("transparent", rgb, [0.0, 0.0, 0.0], 0.0),
            ("#f00", rgb, [1.0, 0.0, 0.0], 1.0),
            ("red", rgb, [1.0, 0.0, 0.0], 1.0),
            ("CanvasText", rgb, [1.0, 0.0, 0.0], 1.0),
            ("rgb(255 0 0)", rgb, [1.0, 0.0, 0.0], 1.0),
            ("hsl(0 100% 50%)", rgb, [1.0, 0.0, 0.0], 1.0),
            ("hwb(0 0% 0%)", rgb, [1.0, 0.0, 0.0], 1.0),
            ("lab(50 0 0)", ColorSpace::Lab, [50.0, 0.0, 0.0], 1.0),
            ("lch(50 0 0)", ColorSpace::Lch, [50.0, 0.0, 0.0], 1.0),
            ("oklab(.5 0 0)", ColorSpace::Oklab, [0.5, 0.0, 0.0], 1.0),
            ("oklch(.5 0 0)", ColorSpace::Oklch, [0.5, 0.0, 0.0], 1.0),
            ("color(srgb 1 0 0)", rgb, [1.0, 0.0, 0.0], 1.0),
            ("alpha(from red / .5)", rgb, [1.0, 0.0, 0.0], 0.5),
            ("rgb(from red calc(r / 2) g b)", rgb, [0.5, 0.0, 0.0], 1.0),
            (
                "color-mix(in srgb, red 25%, blue 75%)",
                rgb,
                [0.25, 0.0, 0.75],
                1.0,
            ),
            (
                "light-dark(rgb(calc(10 + 20) 0 0), currentcolor)",
                rgb,
                [30.0 / 255.0, 0.0, 0.0],
                1.0,
            ),
            ("contrast-color(white)", rgb, [0.0, 0.0, 0.0], 1.0),
            ("device-cmyk(0 1 1 0)", rgb, [1.0, 0.0, 0.0], 1.0),
            (
                "light-dark(red, rgb(0 0 calc(2 + 3",
                rgb,
                [1.0, 0.0, 0.0],
                1.0,
            ),
        ] {
            let color = parsed(source);
            let actual =
                resolve(&color, &facts).unwrap_or_else(|error| panic!("{source}: {error:?}"));
            assert_eq!(actual.space(), space, "{source}");
            assert_eq!(actual.channels(), channels.map(Some), "{source}");
            assert_eq!(actual.alpha(), Some(alpha), "{source}");
        }
        let profile = crate::CssColorProfileName::try_new("--p").unwrap();
        let resource = CssColorFactIdentity::new(8, 9);
        let intent = crate::CssColorProfileRenderingIntent::RelativeColorimetric;
        let names = [crate::CssColorProfileComponentName::try_new("x").unwrap()];
        let bindings = [crate::CssColorProfileBinding::new(
            &profile,
            resource,
            crate::CssColorProfileAvailability::Loaded,
            intent,
            5,
            &names,
        )];
        let inputs = [0.1, 0.2, 0.3, 0.4, 0.5];
        let target = ColorSpace::Predefined(PredefinedColorSpace::XyzD50);
        let converted = Color::new(
            target,
            [
                surgeist_color::Finite::new(0.2),
                surgeist_color::Finite::new(0.3),
                surgeist_color::Finite::new(0.4),
            ],
            surgeist_color::UnitInterval::new(1.0),
        );
        let conversions = [crate::CssColorProfileConversion::Resolved(
            crate::CssColorProfileConversionRecord::new(
                resource, &inputs, intent, target, converted,
            ),
        )];
        let facts = CssColorFacts::new(
            id,
            CssColorPropertyFacts::Unavailable,
            CssColorSchemeFact::Unavailable,
            &[],
            &bindings,
            &conversions,
            CssDeviceColorFact::Unknown,
        );
        for source in [
            "color(--p .1 .2 .3 .4 .5 / .5)",
            "color(from red --p .1 .2 .3 .4 .5 / .5)",
        ] {
            let color = parsed(source);
            let actual =
                resolve(&color, &facts).unwrap_or_else(|error| panic!("{source}: {error:?}"));
            assert_eq!(actual.space(), target);
            assert_eq!(actual.channels(), [Some(0.2), Some(0.3), Some(0.4)]);
            assert_eq!(actual.alpha(), Some(0.5));
        }
        let relative = parsed("color(from red --p x .2 .3 .4 .5)");
        assert!(matches!(
            resolve(&relative, &facts),
            Err(core::Error::Resolution(
                crate::color::resolution::Error::ReverseProfileUnsupported
            ))
        ));
    }
}
