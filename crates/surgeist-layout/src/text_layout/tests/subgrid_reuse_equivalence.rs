//! Reusing a nested subgrid computation leaves the layout output unchanged.
//!
//! Each document lays out twice through `compute_layout_with_flow`, once as the layout runs and
//! once with every nested subgrid computation repeated, and the two completed batches must be
//! identical: the cache stores and clears in order, the node layouts and everything else the
//! batch publishes. The documents run the multi-pass text and group profiles that replay a
//! layout, in which a staged effect of the first pass must not outlive its pass.

use super::*;
use crate::grid::subgrid_work::{SubgridWork, take_subgrid_work, with_subgrid_reuse_disabled};
use crate::{
    ChildOrderCompleteness, GeneratedBoxRef, GridPlacement, InlineChildOrderInput,
    NativeInlineBoxFamilyInput, SubgridTrack, TextGroupAlign, TrackComponentOf,
};

/// What the innermost subgrid holds.
#[derive(Clone, Copy)]
enum Content {
    /// A fixed Block beside a fixed Block, with no text and no group.
    Plain,
    /// A Block of text beside a fixed Block, so intrinsic sizing flows text through every level.
    Text,
    /// A group-aligned Block beside a fixed Block.
    Group,
    /// A group-aligned Block of text beside a fixed Block.
    GroupedText,
    /// Two fixed Blocks that are native inline-box formatters.
    Formatter,
}

/// Where the content sits.
#[derive(Clone, Copy)]
enum Shape {
    /// In the innermost of subgrids that inherit both axes, so no level has tracks of its own.
    Inherited,
    /// In each level of subgrids that inherit the columns and size their own rows from content.
    OwnRows,
}

impl Content {
    const fn has_group(self) -> bool {
        matches!(self, Self::Group | Self::GroupedText)
    }
}

/// The wrapping input that registers each of `formatters` as a native inline-box family.
fn native_formatters(formatters: &[u32]) -> crate::WrappingInput<u32> {
    crate::WrappingInput {
        native_inline_boxes: formatters
            .iter()
            .map(|&formatter| NativeInlineBoxFamilyInput { formatter })
            .collect(),
        gaps: Vec::new(),
        scopes: Vec::new(),
        child_order: formatters
            .iter()
            .map(|&formatter| InlineChildOrderInput {
                parent: GeneratedBoxRef::Node(formatter),
                children: Vec::new(),
                prefix: ChildOrderCompleteness::Complete,
                suffix: ChildOrderCompleteness::Complete,
                formatting_context: formatter,
            })
            .collect(),
    }
}

fn subgrid_tracks<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![TrackComponentOf::Subgrid(SubgridTrack::new(Vec::new()))]
}

fn auto_tracks<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO]
}

fn px<S: LayoutScalar>(width: f64, height: f64) -> Size<PreferredSizeOf<S>> {
    Size::new(
        PreferredSizeOf::px(S::from_f64(width)),
        PreferredSizeOf::px(S::from_f64(height)),
    )
}

// Node 0 is a Grid with two automatic columns and two automatic rows. Nodes 1..=depth are Grid
// subgrids in both axes, each the only item of its parent and spanning both tracks of each axis.
// The innermost holds two Blocks, `depth + 1` in the first track of each axis and `depth + 2` in
// the second. For text, `depth + 3` is the text child of the first Block.
fn lay_out<S: LayoutScalar>(depth: u32, content: Content, width: f64) -> String {
    let text_node = depth + 3;
    let has_text = matches!(content, Content::Text | Content::GroupedText);
    let paragraph = has_text.then(|| {
        PreparedParagraph::plain(
            text_node,
            Source::new("XX XXXX XX XXX XX XXXXX X"),
            ahem_style(10.0, 40.0),
        )
    });
    let mut tree = PublicLayoutTreeOf::<S>::new().style(
        0,
        NodeInputOf {
            display: Display::Grid,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(width)),
                PreferredSizeOf::AUTO,
            ),
            grid_template_columns: auto_tracks(),
            grid_template_rows: auto_tracks(),
            ..NodeInputOf::default()
        },
    );
    for node in 1..=depth {
        tree = tree
            .style(
                node,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_row: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_template_columns: subgrid_tracks(),
                    grid_template_rows: subgrid_tracks(),
                    ..NodeInputOf::default()
                },
            )
            .children(node - 1, [node]);
    }
    let group = match content {
        Content::Plain | Content::Text | Content::Formatter => TextGroupAlign::None,
        Content::Group | Content::GroupedText => TextGroupAlign::Center,
    };
    tree = tree
        .style(
            depth + 1,
            NodeInputOf {
                display: Display::Block,
                text_group_align: group,
                size: if has_text {
                    Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
                } else {
                    px(30.0, 20.0)
                },
                grid_column: GridPlacement::try_line(1).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            depth + 2,
            NodeInputOf {
                display: Display::Block,
                size: px(40.0, 50.0),
                grid_column: GridPlacement::try_line(2).unwrap(),
                grid_row: GridPlacement::try_line(2).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .children(depth, [depth + 1, depth + 2])
        .children(depth + 2, []);
    let mut bindings = Vec::new();
    if let Some(paragraph) = &paragraph {
        tree = tree
            .children(depth + 1, [text_node])
            .children(text_node, [])
            .input(text_node, paragraph.text_input(text_node));
        bindings.push(paragraph.binding(depth + 1));
    } else {
        tree = tree.children(depth + 1, []);
    }
    let wrapping = match content {
        Content::Formatter => native_formatters(&[depth + 1, depth + 2]),
        _ => crate::WrappingInput::default(),
    };
    complete_flow(&tree, bindings, wrapping, width)
}

// Node 0 is a Grid with two automatic columns and an automatic row. Nodes 1..=depth are Grid
// subgrids in the column axis only, each with two automatic rows of its own: the next subgrid
// in the first row and a Block `depth + level` in the second, so each level sizes its own rows
// from content. The second Block holds text, a group alignment, or neither (a fixed size).
// For text, `2 * depth + level` is the text child of the Block of that level.
fn lay_out_levels<S: LayoutScalar>(depth: u32, content: Content, width: f64) -> String {
    let (tree, bindings, wrapping) = levels_document::<S>(depth, content, width);
    complete_flow(&tree, bindings, wrapping, width)
}

type Document<S> = (
    PublicLayoutTreeOf<S>,
    Vec<TextParagraphBinding<u32>>,
    crate::WrappingInput<u32>,
);

fn levels_document<S: LayoutScalar>(depth: u32, content: Content, width: f64) -> Document<S> {
    let has_text = matches!(content, Content::Text | Content::GroupedText);
    let group = match content {
        Content::Plain | Content::Text | Content::Formatter => TextGroupAlign::None,
        Content::Group | Content::GroupedText => TextGroupAlign::Center,
    };
    let mut tree = PublicLayoutTreeOf::<S>::new().children(0, [1]).style(
        0,
        NodeInputOf {
            display: Display::Grid,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(width)),
                PreferredSizeOf::AUTO,
            ),
            grid_template_columns: auto_tracks(),
            grid_template_rows: vec![TrackComponentOf::AUTO],
            ..NodeInputOf::default()
        },
    );
    let mut bindings = Vec::new();
    for level in 1..=depth {
        let block = depth + level;
        let mut children = if level < depth {
            vec![level + 1]
        } else {
            Vec::new()
        };
        children.push(block);
        tree = tree
            .style(
                level,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_row: GridPlacement::try_line(1).unwrap(),
                    grid_template_columns: subgrid_tracks(),
                    grid_template_rows: auto_tracks(),
                    ..NodeInputOf::default()
                },
            )
            .children(level, children)
            .style(
                block,
                NodeInputOf {
                    display: Display::Block,
                    text_group_align: group,
                    size: if has_text {
                        Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
                    } else {
                        px(30.0, 20.0)
                    },
                    grid_column: GridPlacement::try_line(1).unwrap(),
                    grid_row: GridPlacement::try_line(2).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        if has_text {
            let text_node = 2 * depth + level;
            let paragraph = PreparedParagraph::plain(
                text_node,
                Source::new("XX XXXX XX XXX XX XXXXX X"),
                ahem_style(10.0, 40.0),
            );
            tree = tree
                .children(block, [text_node])
                .children(text_node, [])
                .input(text_node, paragraph.text_input(text_node));
            bindings.push(paragraph.binding(block));
        } else {
            tree = tree.children(block, []);
        }
    }
    let wrapping = match content {
        Content::Formatter => native_formatters(&(depth + 1..=2 * depth).collect::<Vec<_>>()),
        _ => crate::WrappingInput::default(),
    };
    (tree, bindings, wrapping)
}

fn complete_flow<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    bindings: Vec<TextParagraphBinding<u32>>,
    wrapping: crate::WrappingInput<u32>,
    width: f64,
) -> String {
    match compute_flow(tree, bindings, wrapping, width) {
        Ok(batch) => format!("{batch:?}"),
        Err(error) => format!("refused: {error:?}"),
    }
}

fn compute_flow<Tree, S>(
    tree: &Tree,
    bindings: Vec<TextParagraphBinding<u32>>,
    wrapping: crate::WrappingInput<u32>,
    width: f64,
) -> Result<CompletedLayoutBatchOf<u32, S>, crate::LayoutErrorOf<u32, S, Tree::MeasureError>>
where
    S: LayoutScalar,
    Tree: LayoutTree<Node = u32, Scalar = S>,
    Tree::MeasureError: core::fmt::Debug,
{
    let capture = LayoutFlowInput::try_new(tree, 0, bindings, wrapping, Vec::new()).unwrap();
    compute_layout_with_flow(
        &capture,
        0,
        request(width, {
            let root_input = crate::LayoutTree::node_input(capture.tree(), 0);
            crate::FlowAxes::new(root_input.writing_mode, root_input.direction)
        }),
    )
}

/// A host whose cache holds what an earlier layout stored.
struct CachedHost<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    cache: std::cell::RefCell<Vec<crate::LayoutCacheStoreEntryOf<u32, S>>>,
    lookups: std::cell::Cell<usize>,
}

impl<S: LayoutScalar> Traverse for CachedHost<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for CachedHost<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }

    fn has_leaf_measurement(&self, node: u32) -> bool {
        self.tree.has_leaf_measurement(node)
    }

    fn measure_leaf(
        &self,
        node: u32,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        self.tree.measure_leaf(node, input)
    }

    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        self.lookups.set(self.lookups.get() + 1);
        self.cache
            .borrow()
            .iter()
            .find(|entry| {
                entry.node() == node && entry.input() == input && entry.context() == context
            })
            .map(|entry| entry.output())
    }
}

/// The debug rendering of a cold layout and of the layout that follows it on a host whose cache
/// holds what the cold layout stored, and how many lookups the warm layout made of it.
fn lay_out_levels_on_a_warm_host<S: LayoutScalar>(
    depth: u32,
    content: Content,
    width: f64,
) -> (String, String, usize) {
    let (tree, bindings, wrapping) = levels_document::<S>(depth, content, width);
    let host = CachedHost {
        tree,
        cache: std::cell::RefCell::new(Vec::new()),
        lookups: std::cell::Cell::new(0),
    };
    let render = |batch: &CompletedLayoutBatchOf<u32, S>| format!("{batch:?}");
    // A flow layout leaves the host cache unconsulted, so a document without text or native
    // formatters lays out as a plain tree, which does consult it.
    let compute = |host: &CachedHost<S>| {
        if bindings.is_empty() && wrapping == crate::WrappingInput::default() {
            compute_layout(
                host,
                0,
                request(width, {
                    let root_input = crate::LayoutTree::node_input(host, 0);
                    crate::FlowAxes::new(root_input.writing_mode, root_input.direction)
                }),
            )
            .unwrap()
        } else {
            compute_flow(host, bindings.clone(), wrapping.clone(), width).unwrap()
        }
    };
    let cold = compute(&host);
    host.cache
        .borrow_mut()
        .extend(cold.cache_store_entries().iter().copied());
    host.lookups.set(0);
    let warm = compute(&host);
    (render(&cold), render(&warm), host.lookups.get())
}

/// Lays out one document with the reuse and without it, and returns the work each did.
fn assert_same_without_reuse<S: LayoutScalar>(
    label: &str,
    shape: Shape,
    depth: u32,
    content: Content,
    width: f64,
) -> (SubgridWork, SubgridWork) {
    let document = || match shape {
        Shape::Inherited => lay_out::<S>(depth, content, width),
        Shape::OwnRows => lay_out_levels::<S>(depth, content, width),
    };
    take_subgrid_work();
    let reused = document();
    let reused_work = take_subgrid_work();
    let repeated = with_subgrid_reuse_disabled(document);
    let repeated_work = take_subgrid_work();
    assert!(
        !reused.starts_with("refused"),
        "{label}: the document lays out: {reused:.300}"
    );
    if reused != repeated {
        let at = reused
            .chars()
            .zip(repeated.chars())
            .position(|(left, right)| left != right)
            .unwrap_or_else(|| reused.len().min(repeated.len()));
        let window = |text: &str| {
            text.chars()
                .skip(at.saturating_sub(150))
                .take(400)
                .collect::<String>()
        };
        panic!(
            "{label}: reuse changed the layout at depth {depth}, first at {at}: reused {} repeated {}",
            window(&reused),
            window(&repeated)
        );
    }
    (reused_work, repeated_work)
}

fn assert_same_in_both_scalars(label: &'static str, shape: Shape, content: Content) {
    // The repeated layout recurses once per path through the chain, so it needs a large stack.
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(move || assert_same_at_depths(label, shape, content))
        .unwrap()
        .join()
        .unwrap();
}

fn assert_same_at_depths(label: &str, shape: Shape, content: Content) {
    // Repeating a level that sizes its own rows costs exponentially in the depth.
    let depths: &[u32] = match shape {
        Shape::Inherited => &[2, 3],
        Shape::OwnRows => &[2],
    };
    for &depth in depths {
        for width in [90.0, 140.0] {
            let (reused, repeated) =
                assert_same_without_reuse::<f32>(label, shape, depth, content, width);
            if content.has_group() {
                // Group alignment numbers its owners and members by the sequence of
                // computations, so a group session skips no baseline refresh.
                assert_eq!(
                    reused.baseline_refreshes, repeated.baseline_refreshes,
                    "{label}: a group session reuses nothing at depth {depth}: {reused:?} against {repeated:?}"
                );
            } else {
                assert!(
                    reused.wrapper_preparations < repeated.wrapper_preparations,
                    "{label}: the reuse engages at depth {depth}: {reused:?} against {repeated:?}"
                );
                assert!(
                    reused.baseline_refreshes < repeated.baseline_refreshes,
                    "{label}: the baseline refresh is skipped at depth {depth}: {reused:?} against {repeated:?}"
                );
            }
            assert_same_without_reuse::<f64>(label, shape, depth, content, width);
        }
    }
}

#[test]
fn subgrid_wrappers_holding_text_lay_out_the_same_with_reuse() {
    assert_same_in_both_scalars("text", Shape::Inherited, Content::Text);
}

#[test]
fn subgrid_wrappers_holding_a_group_lay_out_the_same_with_reuse() {
    assert_same_in_both_scalars("group", Shape::Inherited, Content::Group);
}

#[test]
fn subgrid_wrappers_holding_grouped_text_lay_out_the_same_with_reuse() {
    assert_same_in_both_scalars("grouped text", Shape::Inherited, Content::GroupedText);
}

#[test]
fn subgrid_wrappers_with_own_rows_holding_text_lay_out_the_same_with_reuse() {
    assert_same_in_both_scalars("own rows, text", Shape::OwnRows, Content::Text);
}

#[test]
fn subgrid_wrappers_with_own_rows_holding_a_group_lay_out_the_same_with_reuse() {
    assert_same_in_both_scalars("own rows, group", Shape::OwnRows, Content::Group);
}

#[test]
fn subgrid_wrappers_with_own_rows_holding_grouped_text_lay_out_the_same_with_reuse() {
    assert_same_in_both_scalars(
        "own rows, grouped text",
        Shape::OwnRows,
        Content::GroupedText,
    );
}

#[test]
fn subgrid_wrappers_with_own_rows_holding_fixed_blocks_lay_out_the_same_with_reuse() {
    assert_same_in_both_scalars("own rows, fixed", Shape::OwnRows, Content::Plain);
}

#[test]
fn subgrid_wrappers_holding_native_formatters_lay_out_the_same_with_reuse() {
    assert_same_in_both_scalars("native formatters", Shape::Inherited, Content::Formatter);
}

#[test]
fn subgrid_wrappers_with_own_rows_holding_native_formatters_lay_out_the_same_with_reuse() {
    assert_same_in_both_scalars(
        "own rows, native formatters",
        Shape::OwnRows,
        Content::Formatter,
    );
}

// A Grid with two columns holds a subgrid in the first, which holds a native formatter, and a
// Block that is a native formatter in the second. The subgrid is placed and its first
// computation completes the first family; the Block completes the second; the baseline refresh
// then computes the subgrid again, which completes the first family again. A family that
// completes again is published after the families that completed since.
fn lay_out_formatter_beside_subgrid<S: LayoutScalar>() -> String {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 3])
        .children(1, [2])
        .children(2, [])
        .children(3, [])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::AUTO,
                ),
                grid_template_columns: auto_tracks(),
                grid_template_rows: vec![TrackComponentOf::AUTO],
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_line(1).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                grid_template_columns: subgrid_tracks(),
                grid_template_rows: subgrid_tracks(),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                size: px(30.0, 20.0),
                grid_column: GridPlacement::try_line(1).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Block,
                size: px(40.0, 20.0),
                grid_column: GridPlacement::try_line(2).unwrap(),
                grid_row: GridPlacement::try_line(1).unwrap(),
                ..NodeInputOf::default()
            },
        );
    complete_flow(&tree, Vec::new(), native_formatters(&[2, 3]), 100.0)
}

#[test]
fn native_formatter_families_publish_in_the_same_order_with_reuse() {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(|| {
            for (reused, repeated) in [
                (
                    lay_out_formatter_beside_subgrid::<f32>(),
                    with_subgrid_reuse_disabled(lay_out_formatter_beside_subgrid::<f32>),
                ),
                (
                    lay_out_formatter_beside_subgrid::<f64>(),
                    with_subgrid_reuse_disabled(lay_out_formatter_beside_subgrid::<f64>),
                ),
            ] {
                assert!(!reused.starts_with("refused"), "the document lays out: {reused:.300}");
                assert!(
                    reused == repeated,
                    "reuse changed the order of the native families: reused {reused:.600} repeated {repeated:.600}"
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn subgrid_wrappers_lay_out_the_same_with_reuse_on_a_warm_host_cache() {
    std::thread::Builder::new()
        .stack_size(256 * 1024 * 1024)
        .spawn(|| {
            for content in [Content::Plain, Content::Text, Content::Formatter] {
                let reused = lay_out_levels_on_a_warm_host::<f64>(2, content, 140.0);
                let repeated = with_subgrid_reuse_disabled(|| {
                    lay_out_levels_on_a_warm_host::<f64>(2, content, 140.0)
                });
                assert!(
                    reused.0 == repeated.0,
                    "the cold layout is the same with reuse"
                );
                assert!(
                    reused.1 == repeated.1,
                    "the warm layout is the same with reuse"
                );
                if matches!(content, Content::Plain) {
                    assert!(reused.2 > 0, "the warm layout consults the host cache");
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
