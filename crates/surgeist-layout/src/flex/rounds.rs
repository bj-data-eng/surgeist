use super::*;

pub(super) type ResolvedFlexRound<Node, S> = (
    Constants<S>,
    Vec<ResolvedFlexItem<Node, S>>,
    Vec<FlexLine<S>>,
);

type FlexRoundResult<Node, S, M> = LayoutResultOf<Node, ResolvedFlexRound<Node, S>, S, M>;

pub(super) enum FlexRoundChild<Node, S: LayoutScalar> {
    Recollect(FlexItemRequest<Node, S>),
    Intrinsic(FlexItemRequest<Node, S>),
    CrossSize(CrossSizeChild<Node, S>),
}

impl<Node: Copy, S: LayoutScalar> FlexRoundChild<Node, S> {
    pub(super) fn request(&self) -> (Node, ComputeInputOf<S>) {
        let request = match self {
            Self::Recollect(request) | Self::Intrinsic(request) => request,
            Self::CrossSize(child) => &child.request,
        };
        (request.node, request.input)
    }
}

type FlexRoundsStep<Node, S, M> =
    LayoutResultOf<Node, ControlFlow<ResolvedFlexRound<Node, S>, FlexRoundChild<Node, S>>, S, M>;

type FlexRoundStart<'input, Node, S, M> =
    LayoutResultOf<Node, (Constants<S>, RoundPhase<'input, Node, S>), S, M>;

pub(super) struct FlexRounds<'input, Node, S: LayoutScalar> {
    base: Constants<S>,
    run_mode: RunMode,
    capture_struts: bool,
    constants: Constants<S>,
    recollected: Option<Vec<CollectedFlexItem<Node, S>>>,
    phase: Option<RoundPhase<'input, Node, S>>,
}

enum RoundPhase<'input, Node, S: LayoutScalar> {
    Recollect {
        items: ItemCollection<'input, Node, S>,
        struts: Option<CollapsedFlexStruts<Node, S>>,
    },
    Intrinsic {
        round: FlexRound<Node, S>,
        measurement: ControlFlow<S, IntrinsicMainSize<S>>,
    },
    CrossSize(FlexRoundCrossSize<Node, S>),
}

impl<'input, Node: Copy + Eq, S: LayoutScalar> FlexRounds<'input, Node, S> {
    pub(super) fn begin<M>(
        node: Node,
        original: &[CollectedFlexItem<Node, S>],
        base: Constants<S>,
        run_mode: RunMode,
    ) -> LayoutResultOf<Node, Self, S, M> {
        let capture_struts = original.iter().any(CollectedFlexItem::is_collapsed);
        let (constants, phase) = Self::begin_round(node, original, base, run_mode, None)?;
        Ok(Self {
            base,
            run_mode,
            capture_struts,
            constants,
            recollected: None,
            phase: Some(phase),
        })
    }

    fn begin_round<M>(
        node: Node,
        original: &[CollectedFlexItem<Node, S>],
        mut constants: Constants<S>,
        run_mode: RunMode,
        struts: Option<CollapsedFlexStruts<Node, S>>,
    ) -> FlexRoundStart<'input, Node, S, M> {
        let round = if struts.is_some() {
            FlexLineCollectionRound::Collapsed
        } else {
            FlexLineCollectionRound::Normal
        };
        let phase = if let Some(offer) = column_cross_offer(&constants, original, round) {
            // The common cross offer can change flex bases as well as line widths.
            constants.cross_content_offer = Some(offer.max(S::ZERO));
            RoundPhase::Recollect {
                items: ItemCollection::recollect(original.len(), run_mode, round),
                struts,
            }
        } else {
            Self::begin_main_size(node, &constants, original, struts)?
        };
        Ok((constants, phase))
    }

    fn begin_main_size<M>(
        node: Node,
        constants: &Constants<S>,
        items: &[CollectedFlexItem<Node, S>],
        struts: Option<CollapsedFlexStruts<Node, S>>,
    ) -> LayoutResultOf<Node, RoundPhase<'input, Node, S>, S, M> {
        let round = FlexRound::begin(node, *constants, items, struts)?;
        let measurement =
            begin_main_size(&round.constants, round.items(items), round.sizing_lines());
        Ok(RoundPhase::Intrinsic { round, measurement })
    }

    pub(super) fn advance<Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        original: &[CollectedFlexItem<Node, S>],
    ) -> FlexRoundsStep<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            match self.phase.as_mut().expect("active Flex round") {
                RoundPhase::Recollect { items, .. } => {
                    if let Some(child) = items.next_recollected(tree, &self.constants, original)? {
                        return Ok(ControlFlow::Continue(FlexRoundChild::Recollect(child)));
                    }
                }
                RoundPhase::Intrinsic { round, measurement } => {
                    let items = self.recollected.as_deref().unwrap_or(original);
                    if let ControlFlow::Continue(intrinsic) = measurement
                        && let Some(child) =
                            intrinsic.next(tree, &round.constants, round.items(items))?
                    {
                        return Ok(ControlFlow::Continue(FlexRoundChild::Intrinsic(child)));
                    }
                }
                RoundPhase::CrossSize(cross) => {
                    let items = self.recollected.as_deref().unwrap_or(original);
                    if let Some(child) = cross.advance(tree, node, items)? {
                        return Ok(ControlFlow::Continue(FlexRoundChild::CrossSize(child)));
                    }
                }
            }
            let phase = self.phase.take().expect("active Flex round");
            self.phase = Some(match phase {
                RoundPhase::Recollect { items, struts } => {
                    let items = items.finish();
                    let phase = Self::begin_main_size(node, &self.constants, &items, struts)?;
                    self.recollected = Some(items);
                    phase
                }
                RoundPhase::Intrinsic { round, measurement } => {
                    let outer_size = match measurement {
                        ControlFlow::Break(size) => size,
                        ControlFlow::Continue(intrinsic) => intrinsic.outer_size(&round.constants),
                    };
                    let items = self.recollected.as_deref().unwrap_or(original);
                    RoundPhase::CrossSize(round.begin_cross_size(tree, node, items, outer_size)?)
                }
                RoundPhase::CrossSize(cross) => {
                    let layout = cross.finish();
                    if !self.capture_struts {
                        return Ok(ControlFlow::Break(layout));
                    }
                    let struts = CollapsedFlexStruts::capture(original, &layout.2);
                    self.capture_struts = false;
                    self.recollected = None;
                    let (constants, phase) =
                        Self::begin_round(node, original, self.base, self.run_mode, Some(struts))?;
                    self.constants = constants;
                    phase
                }
            });
        }
    }

    pub(super) fn accept<Tree, M>(
        &mut self,
        tree: &Tree,
        node: Node,
        original: &[CollectedFlexItem<Node, S>],
        child: FlexRoundChild<Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        match (self.phase.as_mut(), child) {
            (Some(RoundPhase::Recollect { items, .. }), FlexRoundChild::Recollect(_)) => {
                items.accept(tree, &self.constants, output)
            }
            (
                Some(RoundPhase::Intrinsic {
                    round,
                    measurement: ControlFlow::Continue(intrinsic),
                }),
                FlexRoundChild::Intrinsic(_),
            ) => {
                let items = self.recollected.as_deref().unwrap_or(original);
                intrinsic.accept(tree, &round.constants, round.items(items), output)
            }
            (Some(RoundPhase::CrossSize(cross)), FlexRoundChild::CrossSize(child)) => {
                cross.accept(node, child, output)
            }
            _ => unreachable!("Flex child receipt resumes its owning round phase"),
        }
    }

    pub(super) fn compute<Tree, M>(
        mut self,
        tree: &mut Tree,
        node: Node,
        original: &[CollectedFlexItem<Node, S>],
    ) -> FlexRoundResult<Node, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        loop {
            match self.advance(tree, node, original)? {
                ControlFlow::Break(layout) => return Ok(layout),
                ControlFlow::Continue(child) => {
                    let (child_node, input) = child.request();
                    let output = tree.compute_child(child_node, input)?;
                    self.accept(tree, node, original, child, output)?;
                }
            }
        }
    }
}

fn column_cross_offer<Node, S: LayoutScalar>(
    constants: &Constants<S>,
    collected_items: &[CollectedFlexItem<Node, S>],
    round: FlexLineCollectionRound,
) -> Option<S> {
    if constants.is_wrapping_column()
        && constants
            .axes
            .cross_size(constants.node_outer_size)
            .is_none()
        && constants
            .axes
            .cross_size(constants.sizing_request.axes())
            .operation()
            == crate::SizingOperation::MaxContent
    {
        collected_items
            .iter()
            .filter(|item| round == FlexLineCollectionRound::Normal || !item.is_collapsed())
            .map(|item| {
                constants.axes.cross_size(item.hypothetical_size)
                    + constants.axes.cross_edge_sum(item.margin)
            })
            .reduce(S::max)
    } else {
        None
    }
}

type CollapsedFlexItems<Node, S> = (Vec<CollectedFlexItem<Node, S>>, Vec<FlexLine<S>>);

struct FlexRound<Node, S: LayoutScalar> {
    constants: Constants<S>,
    kind: FlexLineCollectionRound,
    lines: Vec<FlexLine<S>>,
    collapsed_struts: Option<CollapsedFlexStruts<Node, S>>,
    collapsed: Option<CollapsedFlexItems<Node, S>>,
}

impl<Node: Copy + Eq, S: LayoutScalar> FlexRound<Node, S> {
    fn begin<M>(
        node: Node,
        constants: Constants<S>,
        collected_items: &[CollectedFlexItem<Node, S>],
        collapsed_struts: Option<CollapsedFlexStruts<Node, S>>,
    ) -> LayoutResultOf<Node, Self, S, M> {
        let kind = if collapsed_struts.is_some() {
            FlexLineCollectionRound::Collapsed
        } else {
            FlexLineCollectionRound::Normal
        };
        let lines = collect_flex_lines(collected_items, &constants, kind)
            .map_err(|reason| crate::inline_wrapping::wrapping_error(node, reason))?;
        let collapsed = collapsed_struts
            .as_ref()
            .map(|struts| struts.prepare_second_round(collected_items, &lines));
        Ok(Self {
            constants,
            kind,
            lines,
            collapsed_struts,
            collapsed,
        })
    }

    fn items<'a>(
        &'a self,
        collected: &'a [CollectedFlexItem<Node, S>],
    ) -> &'a [CollectedFlexItem<Node, S>] {
        self.collapsed
            .as_ref()
            .map_or(collected, |(items, _)| items.as_slice())
    }

    fn sizing_lines(&self) -> &[FlexLine<S>] {
        self.collapsed
            .as_ref()
            .map_or(self.lines.as_slice(), |(_, lines)| lines.as_slice())
    }

    fn begin_cross_size<'input, Tree, M>(
        self,
        tree: &Tree,
        node: Node,
        collected_items: &[CollectedFlexItem<Node, S>],
        outer_size: S,
    ) -> LayoutResultOf<Node, FlexRoundCrossSize<Node, S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let Self {
            constants,
            kind: round,
            mut lines,
            collapsed_struts,
            mut collapsed,
        } = self;
        let layout_constants = finish_main_size(tree, node, constants, outer_size)?;
        if constants.wraps
            && !collected_items.is_empty()
            && flex_line_collection_size(&constants) != flex_line_collection_size(&layout_constants)
        {
            // Intrinsic and auto sizing can use provisional lines. Placement and
            // collapsed struts use the accepted inner main size.
            lines = collect_flex_lines(collected_items, &layout_constants, round)
                .map_err(|reason| crate::inline_wrapping::wrapping_error(node, reason))?;
            collapsed = collapsed_struts
                .as_ref()
                .map(|struts| struts.prepare_second_round(collected_items, &lines));
        }
        let (collapsed_items, lines) = match collapsed {
            Some((items, lines)) => (Some(items), lines),
            None => (None, lines),
        };
        let resolution =
            LineResolution::new(collapsed_items.as_deref().unwrap_or(collected_items), lines);
        Ok(FlexRoundCrossSize {
            constants: layout_constants,
            resolution: Some(resolution),
            collapsed_items,
            followup: None,
            accepted_basis: false,
        })
    }
}

struct FlexRoundCrossSize<Node, S: LayoutScalar> {
    constants: Constants<S>,
    resolution: Option<LineResolution<Node, S>>,
    collapsed_items: Option<Vec<CollectedFlexItem<Node, S>>>,
    followup: Option<CrossSizeChild<Node, S>>,
    accepted_basis: bool,
}

impl<Node: Copy, S: LayoutScalar> FlexRoundCrossSize<Node, S> {
    fn advance<'input, Tree, M>(
        &mut self,
        tree: &mut Tree,
        node: Node,
        collected_items: &[CollectedFlexItem<Node, S>],
    ) -> LayoutResultOf<Node, Option<CrossSizeChild<Node, S>>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        if let Some(child) = self.followup.take() {
            return Ok(Some(child));
        }
        let resolution = self
            .resolution
            .as_mut()
            .expect("active Flex line resolution");
        if let Some(child) = resolution.advance(tree, node, &self.constants)? {
            return Ok(Some(child));
        }
        if !self.accepted_basis {
            let cross_constants =
                resolved_cross_layout_constants(&self.constants, resolution.lines());
            let retry = cross_constants.node_inner_size != self.constants.node_inner_size;
            self.constants = cross_constants;
            self.accepted_basis = true;
            if retry {
                let (_, lines) = self
                    .resolution
                    .take()
                    .expect("first Flex cross sizing")
                    .finish();
                self.resolution = Some(LineResolution::new(
                    self.collapsed_items.as_deref().unwrap_or(collected_items),
                    lines,
                ));
                return self
                    .resolution
                    .as_mut()
                    .expect("accepted Flex cross sizing")
                    .advance(tree, node, &self.constants);
            }
        }
        Ok(None)
    }

    fn accept<M>(
        &mut self,
        node: Node,
        child: CrossSizeChild<Node, S>,
        output: ComputeOutputOf<S>,
    ) -> LayoutResultOf<Node, (), S, M> {
        let resolution = self
            .resolution
            .as_mut()
            .expect("active Flex line resolution");
        self.followup = resolution.accept(node, &self.constants, child, output)?;
        Ok(())
    }

    fn finish(self) -> ResolvedFlexRound<Node, S> {
        let (items, lines) = self
            .resolution
            .expect("completed Flex line resolution")
            .finish();
        (self.constants, items, lines)
    }
}
