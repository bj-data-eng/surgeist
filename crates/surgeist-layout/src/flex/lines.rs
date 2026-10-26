use super::items::CollectedFlexItem;
use super::{AvailableOf, Constants, LayoutScalar};
use crate::inline_wrapping::{AvoidClass, EdgePreferences, WrappingScopeError};

#[derive(Clone, Copy, Debug)]
pub(super) struct FlexLine<S: LayoutScalar> {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) strut_floor: S,
    pub(super) contains_collapsed_slot: bool,
    pub(super) main_size: S,
    pub(super) cross_size: S,
    pub(super) offset_cross: S,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FlexLineCollectionRound {
    Normal,
    Collapsed,
}

#[derive(Clone, Debug)]
pub(super) struct CollapsedFlexStruts<Node, S: LayoutScalar> {
    by_node: Vec<(Node, S)>,
}

impl<Node: Copy + Eq, S: LayoutScalar> CollapsedFlexStruts<Node, S> {
    pub(super) fn capture(items: &[CollectedFlexItem<Node, S>], lines: &[FlexLine<S>]) -> Self {
        let mut by_node = Vec::new();
        for line in lines {
            for item in &items[line.start..line.end] {
                if item.is_collapsed() {
                    by_node.push((item.node, line.cross_size));
                }
            }
        }
        Self { by_node }
    }

    pub(super) fn prepare_second_round(
        &self,
        items: &[CollectedFlexItem<Node, S>],
        collected_lines: &[FlexLine<S>],
    ) -> (Vec<CollectedFlexItem<Node, S>>, Vec<FlexLine<S>>) {
        let mut normal_items = Vec::with_capacity(items.len().saturating_sub(self.by_node.len()));
        let mut lines = Vec::with_capacity(collected_lines.len());
        for collected_line in collected_lines {
            let start = normal_items.len();
            let mut strut_floor = S::ZERO;
            for item in &items[collected_line.start..collected_line.end] {
                if item.is_collapsed() {
                    strut_floor = strut_floor.max(self.for_node(item.node));
                } else {
                    normal_items.push(*item);
                }
            }
            lines.push(FlexLine::with_collapsed_strut(
                start,
                normal_items.len(),
                strut_floor,
            ));
        }
        if lines.is_empty() {
            lines.push(FlexLine::new(0, 0));
        }
        (normal_items, lines)
    }

    fn for_node(&self, node: Node) -> S {
        self.by_node
            .iter()
            .find_map(|(candidate, strut)| (*candidate == node).then_some(*strut))
            .expect("every collapsed flex identity receives one first-round strut")
    }
}

pub(super) fn collect_flex_lines<Node, S: LayoutScalar>(
    items: &[CollectedFlexItem<Node, S>],
    constants: &Constants<S>,
    round: FlexLineCollectionRound,
) -> Result<Vec<FlexLine<S>>, WrappingScopeError>
where
    Node: Copy,
{
    if !constants.wraps {
        return Ok(vec![FlexLine::new(0, items.len())]);
    }

    let container_main_size = match flex_line_collection_size(constants) {
        Some(size) => size,
        None => match constants.available_main {
            AvailableOf::Definite(size) => size,
            AvailableOf::MinContent => {
                return Ok((0..items.len())
                    .map(|index| FlexLine::new(index, index + 1))
                    .collect());
            }
            AvailableOf::MaxContent => {
                let mut lines = Vec::new();
                let mut start = 0;
                for cut in 1..items.len() {
                    if forced_boundary(items, cut) {
                        lines.push(FlexLine::new(start, cut));
                        start = cut;
                    }
                }
                lines.push(FlexLine::new(start, items.len()));
                return Ok(lines);
            }
        },
    };

    let has_avoid = constants.available_main != AvailableOf::MinContent
        && items
            .windows(2)
            .any(|pair| pair[0].wrap_after.avoided || pair[1].wrap_before.avoided);
    let preferences = if has_avoid {
        Some(EdgePreferences::new(
            items
                .len()
                .checked_mul(2)
                .ok_or(WrappingScopeError::InconsistentIncidence)?,
        ))
    } else {
        None
    };
    let mut lines = Vec::new();
    let mut start = 0;
    while start < items.len() {
        let mut line_main_size = S::ZERO;
        let mut end = start;
        let mut frontier = preferences.as_ref().map(EdgePreferences::frontier);

        while end < items.len() {
            if end > start && forced_boundary(items, end) {
                break;
            }
            let gap = if end == start {
                S::ZERO
            } else {
                constants.axes.main_size(constants.gap)
            };
            let item = &items[end];
            let box_main_size =
                if round == FlexLineCollectionRound::Collapsed && item.is_collapsed() {
                    S::ZERO
                } else {
                    constants.axes.main_size(item.hypothetical_size)
                };
            let next_size = gap + box_main_size + constants.axes.main_edge_sum(item.margin);
            if end > start && line_main_size + next_size > container_main_size {
                break;
            }

            line_main_size = line_main_size + next_size;
            end += 1;
            if end < items.len()
                && !forced_boundary(items, end)
                && line_main_size <= container_main_size
                && let (Some(preferences), Some(frontier)) = (&preferences, &mut frontier)
            {
                frontier.consider(boundary_class(items, end, preferences)?, end)?;
            }
        }

        if end < items.len()
            && !forced_boundary(items, end)
            && let Some(preferred) = frontier.and_then(|frontier| frontier.into_preferred())
        {
            end = preferred;
        }
        lines.push(FlexLine::new(start, end));
        start = end;
    }

    if lines.is_empty() {
        lines.push(FlexLine::new(0, 0));
    }
    Ok(lines)
}

fn forced_boundary<Node, S: LayoutScalar>(
    items: &[CollectedFlexItem<Node, S>],
    cut: usize,
) -> bool {
    cut > 0
        && cut < items.len()
        && (items[cut - 1].wrap_after.forced || items[cut].wrap_before.forced)
}

fn boundary_class<Node, S: LayoutScalar>(
    items: &[CollectedFlexItem<Node, S>],
    cut: usize,
    preferences: &EdgePreferences,
) -> Result<AvoidClass, WrappingScopeError> {
    // Each side identifies its actual occurrence in this sorted inventory.
    // Adjacent declarations are independent constraints on their shared cut.
    let mut sides = [0; 2];
    let mut count = 0;
    if items[cut - 1].wrap_after.avoided {
        sides[count] = 2 * (cut - 1) + 1;
        count += 1;
    }
    if items[cut].wrap_before.avoided {
        sides[count] = 2 * cut;
        count += 1;
    }
    preferences.class(&sides[..count])
}

fn flex_line_collection_size<S: LayoutScalar>(constants: &Constants<S>) -> Option<S> {
    constants
        .axes
        .main_size(constants.node_inner_size)
        .or_else(|| constants.axes.main_size(constants.max_inner_size))
}

impl<S: LayoutScalar> FlexLine<S> {
    fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            strut_floor: S::ZERO,
            contains_collapsed_slot: false,
            main_size: S::ZERO,
            cross_size: S::ZERO,
            offset_cross: S::ZERO,
        }
    }

    fn with_collapsed_strut(start: usize, end: usize, strut_floor: S) -> Self {
        Self {
            strut_floor,
            contains_collapsed_slot: true,
            ..Self::new(start, end)
        }
    }
}
