//! Public intrinsic-size qualification at finite, very small available widths.
//! CSS Grid1 §5.2 defines max-content from the fixed track sum; Overflow3
//! §4.1/§4.2 reserves stable classic gutters as border-box space, even for
//! hidden overflow. Available space is not an authored definite border box.
use surgeist_layout::*;

struct FixedGrid<S: LayoutScalar> {
    inputs: [LayoutInputOf<S>; 2],
}

impl<S: LayoutScalar> Traverse for FixedGrid<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = if node == 0 { &[1] } else { &[] };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        usize::from(node == 0)
    }

    fn child(&self, node: usize, index: usize) -> usize {
        assert_eq!((node, index), (0, 0));
        1
    }
}

impl<S: LayoutScalar> LayoutTree for FixedGrid<S> {
    type MeasureError = std::convert::Infallible;

    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap()
    }

    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }
}

fn preferred<S: LayoutScalar>(value: f64) -> PreferredSizeOf<S> {
    PreferredSizeOf::value(LengthPercentageOf::px(S::from_f64(value)).unwrap())
}

fn track<S: LayoutScalar>(value: f64) -> TrackComponentOf<S> {
    let value = SizingCalculationOf::value(LengthPercentageOf::px(S::from_f64(value)).unwrap());
    TrackComponentOf::minmax(
        MinTrackSizingOf::Calculation(value.clone()),
        MaxTrackSizingOf::Calculation(value),
    )
}

fn fixture<S: LayoutScalar>(
    display: Display,
    gutter: ScrollbarGutter,
    width: PreferredSizeOf<S>,
) -> FixedGrid<S> {
    FixedGrid {
        inputs: [
            LayoutInputOf::box_input(NodeInputOf {
                display,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(width, preferred(20.0)),
                grid_template_columns: vec![track(40.0)],
                grid_template_rows: vec![track(20.0)],
                overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap(),
                scrollbar_gutter: gutter,
                scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(5.0)).unwrap(),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            }),
            LayoutInputOf::box_input(NodeInputOf {
                display: Display::Block,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(preferred(40.0), preferred(20.0)),
                ..NodeInputOf::default()
            }),
        ],
    }
}

fn qualify<S: LayoutScalar>(intrinsic: bool) {
    let mut failures = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        for available in [0.0, 2.0] {
            for (gutter, reservation) in [
                (ScrollbarGutter::Auto, 0.0),
                (ScrollbarGutter::Stable, 5.0),
                (ScrollbarGutter::StableBothEdges, 10.0),
            ] {
                let width = if intrinsic {
                    PreferredSizeOf::MAX_CONTENT
                } else {
                    preferred(available)
                };
                let tree = fixture::<S>(display, gutter, width);
                let batch = compute_layout(
                    &tree,
                    0,
                    LayoutRootRequestOf::viewport(Size::new(
                        AvailableOf::Definite(S::from_f64(available)),
                        AvailableOf::MAX_CONTENT,
                    ))
                    .unwrap(),
                )
                .unwrap();
                let root = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == 0)
                    .unwrap()
                    .output();
                let child = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == 1)
                    .unwrap()
                    .output();
                let geometry = root
                    .scroll_geometry
                    .expect("hidden root is a genuine scroll container");
                let expected_width = if intrinsic {
                    40.0 + reservation
                } else {
                    available
                };
                let expected_content_width = if intrinsic {
                    40.0
                } else {
                    (available - reservation).max(0.0)
                };
                let actual = (root.size, child.size, geometry.content_box().size());
                let expected = (
                    Size::new(S::from_f64(expected_width), S::from_f64(20.0)),
                    Size::new(S::from_f64(40.0), S::from_f64(20.0)),
                    Size::new(S::from_f64(expected_content_width), S::from_f64(20.0)),
                );
                eprintln!(
                    "{} {display:?} intrinsic={intrinsic} available={available} {gutter:?}: actual={actual:?}, expected={expected:?}",
                    std::any::type_name::<S>()
                );
                if actual != expected {
                    failures.push(format!("{display:?} available={available} {gutter:?}: actual={actual:?}, expected={expected:?}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn max_content_fixed_grid_gutters_use_intrinsic_box_despite_tiny_available_width_f32() {
    qualify::<f32>(true);
}

#[test]
fn max_content_fixed_grid_gutters_use_intrinsic_box_despite_tiny_available_width_f64() {
    qualify::<f64>(true);
}

#[test]
fn authored_tiny_definite_grid_border_box_still_clamps_gutters_f32() {
    qualify::<f32>(false);
}

#[test]
fn authored_tiny_definite_grid_border_box_still_clamps_gutters_f64() {
    qualify::<f64>(false);
}

struct InheritedColumnGrid<S: LayoutScalar> {
    inputs: [LayoutInputOf<S>; 3],
}

impl<S: LayoutScalar> Traverse for InheritedColumnGrid<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = match node {
            0 => &[1],
            1 => &[2],
            _ => &[],
        };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        usize::from(node < 2)
    }

    fn child(&self, node: usize, index: usize) -> usize {
        assert!(node < 2);
        assert_eq!(index, 0);
        node + 1
    }
}

impl<S: LayoutScalar> LayoutTree for InheritedColumnGrid<S> {
    type MeasureError = std::convert::Infallible;

    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap()
    }

    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }
}

fn qualify_inherited_columns<S: LayoutScalar>() {
    let mut failures = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        for (overflow, gutter, leaf_height, expected_width, reservation) in [
            (Overflow::Auto, ScrollbarGutter::Auto, 20.0, 40.0, 0.0),
            (Overflow::Auto, ScrollbarGutter::Auto, 30.0, 45.0, 5.0),
            (Overflow::Hidden, ScrollbarGutter::Stable, 20.0, 45.0, 5.0),
        ] {
            // Grid2 §9 subgrid-item-contribution/subgrid-margins applies the
            // child's gutter to its edge item's intrinsic contribution. The
            // 40px leaf fills the inherited column. Its 30px block extent
            // overflows the child's independently fixed20 row, requiring5
            // inline gutter. No-overflow Auto and Hidden Stable are controls.
            let tree = InheritedColumnGrid::<S> {
                inputs: [
                    LayoutInputOf::box_input(NodeInputOf {
                        display: Display::Grid,
                        size: Size::new(PreferredSizeOf::MAX_CONTENT, preferred(20.0)),
                        grid_template_columns: vec![TrackComponentOf::MAX_CONTENT],
                        grid_template_rows: vec![track(20.0)],
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    }),
                    LayoutInputOf::box_input(NodeInputOf {
                        display,
                        size: Size::new(PreferredSizeOf::AUTO, preferred(20.0)),
                        grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                            vec![],
                        ))],
                        grid_template_rows: vec![track(20.0)],
                        grid_auto_flow: GridAutoFlow::Row,
                        overflow: ComputedOverflow::try_new(overflow, overflow).unwrap(),
                        scrollbar_gutter: gutter,
                        scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(5.0)).unwrap(),
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    }),
                    LayoutInputOf::box_input(NodeInputOf {
                        display: Display::Block,
                        size: Size::new(preferred(40.0), preferred(leaf_height)),
                        ..NodeInputOf::default()
                    }),
                ],
            };
            let request =
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap();
            let batch = match compute_layout(&tree, 0, request) {
                Ok(batch) => batch,
                Err(error) => {
                    failures.push(format!(
                        "{display:?} {overflow:?} {gutter:?} leaf_height={leaf_height}: {error:?}"
                    ));
                    continue;
                }
            };
            let output = |node| {
                batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == node)
                    .unwrap()
                    .output()
            };
            let parent = output(0);
            let child = output(1);
            let leaf = output(2);
            let geometry = child.scroll_geometry.expect("actual child scroll geometry");
            let actual = (
                parent.size,
                child.size,
                leaf.size,
                geometry.content_box().size(),
                geometry.scrollbar_size(),
            );
            let expected = (
                Size::new(S::from_f64(expected_width), S::from_f64(20.0)),
                Size::new(S::from_f64(expected_width), S::from_f64(20.0)),
                Size::new(S::from_f64(40.0), S::from_f64(leaf_height)),
                Size::new(S::from_f64(40.0), S::from_f64(20.0)),
                Size::new(S::from_f64(reservation), S::ZERO),
            );
            eprintln!(
                "{} inherited_columns {display:?} {overflow:?} {gutter:?} leaf_height={leaf_height}: actual={actual:?}, expected={expected:?}",
                std::any::type_name::<S>()
            );
            if actual != expected {
                failures.push(format!("{display:?} {overflow:?} {gutter:?} leaf_height={leaf_height}: actual={actual:?}, expected={expected:?}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn inherited_columns_include_child_settled_auto_gutter_f32() {
    qualify_inherited_columns::<f32>();
}

#[test]
fn inherited_columns_include_child_settled_auto_gutter_f64() {
    qualify_inherited_columns::<f64>();
}

// Writing Modes4 §6.1: horizontal-tb has horizontal inline/vertical block;
// vertical-* and sideways-* have vertical inline/horizontal block. Only
// sizes are mapped here; no production FlowAxes or direction helper is used.
fn physical_size<T>(writing_mode: WritingMode, inline: T, block: T) -> Size<T> {
    match writing_mode {
        WritingMode::HorizontalTb => Size::new(inline, block),
        WritingMode::VerticalRl
        | WritingMode::VerticalLr
        | WritingMode::SidewaysRl
        | WritingMode::SidewaysLr => Size::new(block, inline),
    }
}

fn qualify_reciprocal_axis_modes<S: LayoutScalar>() {
    let mut failures = Vec::new();
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for columns in [true, false] {
            for display in [Display::Grid, Display::GridLanes] {
                for overflowing in [false, true] {
                    let extra = if overflowing { 5.0 } else { 0.0 };
                    let (inline, block) = if columns { (40.0, 20.0) } else { (20.0, 40.0) };
                    let leaf_inline = if !columns && overflowing {
                        30.0
                    } else {
                        inline
                    };
                    let leaf_block = if columns && overflowing { 30.0 } else { block };
                    let tree = InheritedColumnGrid::<S> {
                        inputs: [
                            LayoutInputOf::box_input(NodeInputOf {
                                display: Display::Grid,
                                writing_mode,
                                size: physical_size(
                                    writing_mode,
                                    if columns {
                                        PreferredSizeOf::MAX_CONTENT
                                    } else {
                                        preferred(inline)
                                    },
                                    if columns {
                                        preferred(block)
                                    } else {
                                        PreferredSizeOf::MAX_CONTENT
                                    },
                                ),
                                grid_template_columns: vec![if columns {
                                    TrackComponentOf::MAX_CONTENT
                                } else {
                                    track(inline)
                                }],
                                grid_template_rows: vec![if columns {
                                    track(block)
                                } else {
                                    TrackComponentOf::MAX_CONTENT
                                }],
                                justify_content: Some(AlignContent::Start),
                                align_content: Some(AlignContent::Start),
                                ..NodeInputOf::default()
                            }),
                            LayoutInputOf::box_input(NodeInputOf {
                                display,
                                writing_mode,
                                size: physical_size(
                                    writing_mode,
                                    if columns {
                                        PreferredSizeOf::AUTO
                                    } else {
                                        preferred(inline)
                                    },
                                    if columns {
                                        preferred(block)
                                    } else {
                                        PreferredSizeOf::AUTO
                                    },
                                ),
                                grid_template_columns: vec![if columns {
                                    TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))
                                } else {
                                    track(inline)
                                }],
                                grid_template_rows: vec![if columns {
                                    track(block)
                                } else {
                                    TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))
                                }],
                                // The inherited dimension is the lanes/grid axis.
                                grid_auto_flow: if columns {
                                    GridAutoFlow::Row
                                } else {
                                    GridAutoFlow::Column
                                },
                                overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto)
                                    .unwrap(),
                                scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(5.0))
                                    .unwrap(),
                                justify_content: Some(AlignContent::Start),
                                align_content: Some(AlignContent::Start),
                                ..NodeInputOf::default()
                            }),
                            LayoutInputOf::box_input(NodeInputOf {
                                display: Display::Block,
                                writing_mode,
                                size: physical_size(
                                    writing_mode,
                                    preferred(leaf_inline),
                                    preferred(leaf_block),
                                ),
                                ..NodeInputOf::default()
                            }),
                        ],
                    };
                    let batch = match compute_layout(
                        &tree,
                        0,
                        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT))
                            .unwrap(),
                    ) {
                        Ok(batch) => batch,
                        Err(error) => {
                            failures.push(format!("{writing_mode:?} columns={columns} {display:?} overflowing={overflowing}: {error:?}"));
                            continue;
                        }
                    };
                    let output = |node| {
                        batch
                            .unrounded_entries()
                            .iter()
                            .find(|entry| entry.node() == node)
                            .unwrap()
                            .output()
                    };
                    let child_geometry = output(1).scroll_geometry.unwrap();
                    let actual = (
                        output(0).size,
                        output(1).size,
                        output(2).size,
                        child_geometry.content_box().size(),
                        child_geometry.scrollbar_size(),
                    );
                    let outer = physical_size(
                        writing_mode,
                        S::from_f64(inline + if columns { extra } else { 0.0 }),
                        S::from_f64(block + if columns { 0.0 } else { extra }),
                    );
                    let expected = (
                        outer,
                        outer,
                        physical_size(
                            writing_mode,
                            S::from_f64(leaf_inline),
                            S::from_f64(leaf_block),
                        ),
                        physical_size(writing_mode, S::from_f64(inline), S::from_f64(block)),
                        physical_size(
                            writing_mode,
                            S::from_f64(if columns { extra } else { 0.0 }),
                            S::from_f64(if columns { 0.0 } else { extra }),
                        ),
                    );
                    eprintln!(
                        "{} reciprocal {writing_mode:?} columns={columns} {display:?} overflowing={overflowing}: actual={actual:?}, expected={expected:?}",
                        std::any::type_name::<S>()
                    );
                    if actual != expected {
                        failures.push(format!("{writing_mode:?} columns={columns} {display:?} overflowing={overflowing}: actual={actual:?}, expected={expected:?}"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn inherited_auto_gutter_tracks_both_axes_and_all_writing_modes_f32() {
    qualify_reciprocal_axis_modes::<f32>();
}

#[test]
fn inherited_auto_gutter_tracks_both_axes_and_all_writing_modes_f64() {
    qualify_reciprocal_axis_modes::<f64>();
}

struct WrappingColumnSubgrid<S: LayoutScalar> {
    boxes: [LayoutInputOf<S>; 3],
    text: [LayoutInputOf<S>; 2],
    non_box: NodeInputOf<S>,
}

impl<S: LayoutScalar> Traverse for WrappingColumnSubgrid<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;
    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = match node {
            0 => &[1],
            1 => &[2],
            2 => &[3, 4],
            _ => &[],
        };
        children.iter().copied()
    }
    fn child_count(&self, node: usize) -> usize {
        self.children(node).count()
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}
impl<S: LayoutScalar> LayoutTree for WrappingColumnSubgrid<S> {
    type MeasureError = std::convert::Infallible;
    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        if node >= 3 {
            &self.non_box
        } else {
            self.boxes[node].as_box().unwrap()
        }
    }
    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        if node >= 3 {
            self.text[node - 3].as_view()
        } else {
            self.boxes[node].as_view()
        }
    }
}

fn prepare_wrapping_text() -> surgeist_text::PreparedText {
    use surgeist_text::*;
    let style = Style {
        font: Font::new().family("Ahem"),
        size: 5.0,
        line_height: LineHeight::Absolute(10.0),
        ..Style::default()
    };
    let first = Source::new("AAAAA\u{200b}");
    let second = Source::new("AAAA");
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            terminal_edge: surgeist_text::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            source: Source::new("AAAAA\u{200b}AAAA"),
            base_style: style.clone(),
            base_direction: ParagraphBaseDirection::LeftToRight,
            segment_breaks: DocumentSegmentBreaks::LineFeedDefault,
            paragraph_strut: ParagraphStrut::DefaultFont,
            origins: vec![
                ParagraphOrigin {
                    text_node: None,
                    source: first,
                    local: Range::new(0, 6),
                    aggregate: Range::new(0, 6),
                    kind: ParagraphOriginKind::Authored,
                },
                ParagraphOrigin {
                    text_node: None,
                    source: second,
                    local: Range::new(0, 4),
                    aggregate: Range::new(6, 10),
                    kind: ParagraphOriginKind::Authored,
                },
            ],
            connections: vec![ParagraphConnection {
                at: 6,
                kind: ParagraphConnectionKind::Continuous,
            }],
            boundary_policies: vec![ParagraphBoundaryPolicy {
                definition: ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: style.wrap,
                following_wrap: style.wrap,
                ancestor_wrap: style.wrap,
            }],
            controls: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        surgeist_text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(
            include_bytes!("../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        )
        .unwrap();
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}

fn qualify_wrapping_min_max<S: LayoutScalar>() {
    let mut failures = Vec::new();
    for minimum in [true, false] {
        // The longest unbreakable segment is25; max-content is25+20=45.
        // At tentativeMIN25 the allowed break yields two10px lines, which
        // overflow the child's fixed15 block size and require5 inline gutter.
        // Thus intrinsicMIN is30. MAX45 fits one10px line and needs no gutter.
        let prepared = prepare_wrapping_text();
        let origins = prepared
            .origin_views()
            .map(|view| view.handle())
            .collect::<Vec<_>>();
        for (origin, source) in origins.iter().zip(["AAAAA\u{200b}", "AAAA"]) {
            assert_eq!(
                origin.view().source().text().code_units(),
                source.encode_utf16().collect::<Vec<_>>()
            );
        }
        let tree = WrappingColumnSubgrid {
            boxes: [
                LayoutInputOf::box_input(NodeInputOf {
                    display: Display::Grid,
                    size: Size::new(
                        if minimum {
                            PreferredSizeOf::MIN_CONTENT
                        } else {
                            PreferredSizeOf::MAX_CONTENT
                        },
                        preferred(15.0),
                    ),
                    grid_template_columns: vec![TrackComponentOf::AUTO],
                    grid_template_rows: vec![track(15.0)],
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                }),
                LayoutInputOf::box_input(NodeInputOf {
                    display: Display::Grid,
                    size: Size::new(PreferredSizeOf::AUTO, preferred(15.0)),
                    grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                        vec![],
                    ))],
                    grid_template_rows: vec![track(15.0)],
                    overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
                    scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(5.0)).unwrap(),
                    justify_content: Some(AlignContent::Start),
                    align_content: Some(AlignContent::Start),
                    ..NodeInputOf::default()
                }),
                LayoutInputOf::box_input(NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                }),
            ],
            text: [
                LayoutInputOf::inline_text(TextParticipation::new(origins[0].clone())),
                LayoutInputOf::inline_text(TextParticipation::new(origins[1].clone())),
            ],
            non_box: NodeInputOf::non_box(),
        };
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![TextParagraphBinding {
                inline_box_fragments: surgeist_layout::InlineBoxFragmentationDemand::NotRequested,
                prepared,
                container: 2,
                nodes: vec![3, 4],
                origins: vec![
                    TextOriginBinding {
                        origin: origins[0].clone(),
                        node: 3,
                    },
                    TextOriginBinding {
                        origin: origins[1].clone(),
                        node: 4,
                    },
                ],
                atomics: vec![],
                controls: vec![],
                line_strut: None,
            }],
            surgeist_layout::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(
            &capture,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
        )
        .unwrap();
        let output = |node| {
            batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
        };
        let child = output(1);
        let geometry = child.scroll_geometry.unwrap();
        let actual = (
            output(0).size,
            child.size,
            output(2).size,
            geometry.content_box().size(),
            geometry.scrollbar_size(),
        );
        let (width, content_width, gutter) = if minimum {
            (30.0, 25.0, 5.0)
        } else {
            (45.0, 45.0, 0.0)
        };
        // Grid1 §6.2/§10: normal alignment stretches this ordinary item to
        // the fixed15 row. Under §6.6 its automatic block minimum is zero:
        // the fixed row has no auto minimum track. The inline fragments below
        // retain their distinct 20/10 line extents, even outside that box.
        let expected = (
            Size::new(S::from_f64(width), S::from_f64(15.0)),
            Size::new(S::from_f64(width), S::from_f64(15.0)),
            Size::new(S::from_f64(content_width), S::from_f64(15.0)),
            Size::new(S::from_f64(content_width), S::from_f64(15.0)),
            Size::new(S::from_f64(gutter), S::ZERO),
        );
        let fragments: Vec<_> = batch
            .unrounded_inline_fragments()
            .iter()
            .filter(|entry| {
                entry.owner() == &surgeist_layout::InlineFragmentOwner::Node(3)
                    || entry.owner() == &surgeist_layout::InlineFragmentOwner::Node(4)
            })
            .map(|entry| {
                let fragment = entry.fragment();
                let finalized = fragment.finalized_fragment();
                let view = finalized.view();
                let surgeist_text::FinalizedFragmentOwner::Origin(origin) = view.owner() else {
                    panic!("authored A/ZWSP units retain their original source owner")
                };
                (
                    match entry.owner() {
                        surgeist_layout::InlineFragmentOwner::Node(node) => *node,
                        surgeist_layout::InlineFragmentOwner::Scope(_) => {
                            panic!("source fixture requires node ownership")
                        }
                    },
                    origin,
                    view.local_range().unwrap(),
                    view.aggregate_range().unwrap(),
                    fragment.line_index(),
                    fragment.baseline(),
                    fragment.rect().unwrap(),
                )
            })
            .collect();
        // Each original A is a real5px unit; ZWSP is a distinct zero-advance
        // original unit. Ahem5 has ascent4/descent1 and LH10, so before6.5
        // rounds to the common baseline7 (17 on MIN's second logical line).
        // Every source fragment receives that same rigid translation: y.5.
        let expected_fragments: Vec<_> = (0..6)
            .map(|index| {
                (
                    3,
                    origins[0].clone(),
                    surgeist_text::Range::new(index, index + 1),
                    surgeist_text::Range::new(index, index + 1),
                    0,
                    Point::new(S::ZERO, S::from_f64(7.0)),
                    ScrollRectOf::try_new(
                        Point::new(S::from_f64(5.0 * index as f64), S::from_f64(0.5)),
                        Size::new(
                            S::from_f64(if index == 5 { 0.0 } else { 5.0 }),
                            S::from_f64(10.0),
                        ),
                    )
                    .unwrap(),
                )
            })
            .chain((0..4).map(|index| {
                (
                    4,
                    origins[1].clone(),
                    surgeist_text::Range::new(index, index + 1),
                    surgeist_text::Range::new(6 + index, 7 + index),
                    usize::from(minimum),
                    Point::new(S::ZERO, S::from_f64(if minimum { 17.0 } else { 7.0 })),
                    ScrollRectOf::try_new(
                        Point::new(
                            S::from_f64((if minimum { 0.0 } else { 25.0 }) + 5.0 * index as f64),
                            S::from_f64(if minimum { 10.5 } else { 0.5 }),
                        ),
                        Size::new(S::from_f64(5.0), S::from_f64(10.0)),
                    )
                    .unwrap(),
                )
            }))
            .collect();
        if actual != expected || fragments != expected_fragments {
            failures.push(format!("minimum={minimum}: actual={actual:?}, expected={expected:?}, fragments={fragments:?}, expected_fragments={expected_fragments:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn inherited_columns_keep_minimum_and_maximum_auto_overflow_sources_distinct_f32() {
    qualify_wrapping_min_max::<f32>();
}

#[test]
fn inherited_columns_keep_minimum_and_maximum_auto_overflow_sources_distinct_f64() {
    qualify_wrapping_min_max::<f64>();
}

fn qualify_max_constrained_alignment<S: LayoutScalar>() {
    let mut failures = Vec::new();
    for display in [Display::Grid, Display::GridLanes] {
        for (known_width, gutter, expected_origin, expected_content) in [
            (false, ScrollbarGutter::Auto, 0.0, 2.0),
            (false, ScrollbarGutter::Stable, 0.0, 0.0),
            (false, ScrollbarGutter::StableBothEdges, 1.0, 0.0),
            (true, ScrollbarGutter::StableBothEdges, 1.0, 0.0),
        ] {
            let mut tree = fixture::<S>(
                display,
                gutter,
                if known_width {
                    preferred(2.0)
                } else {
                    PreferredSizeOf::MAX_CONTENT
                },
            );
            let mut root = tree.node_input(0).clone();
            root.max_size.width =
                MaxSizeOf::value(LengthPercentageOf::px(S::from_f64(2.0)).unwrap());
            tree.inputs[0] = LayoutInputOf::box_input(root);
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
            )
            .unwrap();
            let output = |node| {
                batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == node)
                    .unwrap()
                    .output()
            };
            let root = output(0);
            let child = output(1);
            let content = root.scroll_geometry.unwrap().content_box();
            // The used maximum constrains the actual outer box to2. Under
            // canonical proportional edge saturation, symmetric5+5 gutters
            // become1+1; the fixed first track starts at that actual content
            // origin. A provisional intrinsic inset cannot position children
            // outside the final content box's start (Grid1 §10.5 alignment).
            let actual = (
                root.size,
                child.location,
                child.size,
                content.origin(),
                content.size(),
            );
            let expected = (
                Size::new(S::from_f64(2.0), S::from_f64(20.0)),
                Point::new(S::from_f64(expected_origin), S::ZERO),
                Size::new(S::from_f64(40.0), S::from_f64(20.0)),
                Point::new(S::from_f64(expected_origin), S::ZERO),
                Size::new(S::from_f64(expected_content), S::from_f64(20.0)),
            );
            eprintln!(
                "{} max_constrained {display:?} known_width={known_width} {gutter:?}: actual={actual:?}, expected={expected:?}",
                std::any::type_name::<S>()
            );
            if actual != expected {
                failures.push(format!("{display:?} known_width={known_width} {gutter:?}: actual={actual:?}, expected={expected:?}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn max_constrained_intrinsic_grid_aligns_children_with_final_content_box_f32() {
    qualify_max_constrained_alignment::<f32>();
}

#[test]
fn max_constrained_intrinsic_grid_aligns_children_with_final_content_box_f64() {
    qualify_max_constrained_alignment::<f64>();
}

struct OrthogonalSubgridChain<S: LayoutScalar> {
    inputs: [LayoutInputOf<S>; 5],
}

impl<S: LayoutScalar> Traverse for OrthogonalSubgridChain<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = match node {
            0 => &[1],
            1 => &[2],
            2 => &[3, 4],
            _ => &[],
        };
        children.iter().copied()
    }
    fn child_count(&self, node: usize) -> usize {
        match node {
            0 | 1 => 1,
            2 => 2,
            _ => 0,
        }
    }
    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}

impl<S: LayoutScalar> LayoutTree for OrthogonalSubgridChain<S> {
    type MeasureError = std::convert::Infallible;
    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap()
    }
    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }
}

fn qualify_orthogonal_standalone_contribution<S: LayoutScalar>() {
    let mut failures = Vec::new();
    for parent_mode in [WritingMode::HorizontalTb, WritingMode::VerticalRl] {
        for orthogonal in [false, true] {
            let child_mode = if orthogonal {
                match parent_mode {
                    WritingMode::HorizontalTb => WritingMode::VerticalRl,
                    _ => WritingMode::HorizontalTb,
                }
            } else {
                parent_mode
            };
            let tree = OrthogonalSubgridChain::<S> {
                inputs: [
                    LayoutInputOf::box_input(NodeInputOf {
                        display: Display::Grid,
                        writing_mode: parent_mode,
                        size: Size::new(PreferredSizeOf::MAX_CONTENT, PreferredSizeOf::MAX_CONTENT),
                        grid_template_columns: vec![TrackComponentOf::MAX_CONTENT],
                        grid_template_rows: vec![TrackComponentOf::AUTO],
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    }),
                    LayoutInputOf::box_input(NodeInputOf {
                        display: Display::Grid,
                        writing_mode: parent_mode,
                        grid_template_columns: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                            vec![],
                        ))],
                        grid_template_rows: vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO],
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    }),
                    LayoutInputOf::box_input(NodeInputOf {
                        display: Display::Grid,
                        writing_mode: child_mode,
                        grid_template_columns: vec![if orthogonal {
                            TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))
                        } else {
                            TrackComponentOf::AUTO
                        }],
                        grid_template_rows: vec![if orthogonal {
                            TrackComponentOf::AUTO
                        } else {
                            TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))
                        }],
                        grid_row: GridPlacement::try_span(2).unwrap(),
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    }),
                    LayoutInputOf::box_input(NodeInputOf {
                        display: Display::Block,
                        writing_mode: child_mode,
                        size: physical_size(parent_mode, preferred(40.0), preferred(30.0)),
                        grid_column: GridPlacement::try_line(1).unwrap(),
                        grid_row: GridPlacement::try_line(1).unwrap(),
                        ..NodeInputOf::default()
                    }),
                    LayoutInputOf::box_input(NodeInputOf {
                        display: Display::Block,
                        writing_mode: child_mode,
                        size: physical_size(parent_mode, preferred(60.0), preferred(50.0)),
                        grid_column: GridPlacement::try_line(if orthogonal { 2 } else { 1 })
                            .unwrap(),
                        grid_row: GridPlacement::try_line(if orthogonal { 1 } else { 2 }).unwrap(),
                        ..NodeInputOf::default()
                    }),
                ],
            };
            // Grid2 §12 per-axis recursion: A's column query sees C as an
            // ordinary standalone item in that SAME physical dimension, even
            // though orthogonal C calls it its Row. Fixed leaf contributions
            // share one such track, so max(40,60)=60; the two inherited tracks
            // in the perpendicular dimension sum30+50=80. Writing Modes4 §6.1
            // maps those literal dimensions to60x80 or reciprocal80x60.
            let batch = match compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
            ) {
                Ok(batch) => batch,
                Err(error) => {
                    failures.push(format!(
                        "{parent_mode:?} orthogonal={orthogonal}: {error:?}"
                    ));
                    continue;
                }
            };
            let size = |node| {
                batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == node)
                    .unwrap()
                    .output()
                    .size
            };
            let actual = [size(0), size(1), size(2), size(3), size(4)];
            let outer = physical_size(parent_mode, S::from_f64(60.0), S::from_f64(80.0));
            let expected = [
                outer,
                outer,
                outer,
                physical_size(parent_mode, S::from_f64(40.0), S::from_f64(30.0)),
                physical_size(parent_mode, S::from_f64(60.0), S::from_f64(50.0)),
            ];
            eprintln!(
                "{} orthogonal_standalone {parent_mode:?} orthogonal={orthogonal}: actual={actual:?}, expected={expected:?}",
                std::any::type_name::<S>()
            );
            if actual != expected {
                failures.push(format!("{parent_mode:?} orthogonal={orthogonal}: actual={actual:?}, expected={expected:?}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn nested_subgrid_standalone_contribution_keeps_physical_axis_across_orthogonal_modes_f32() {
    qualify_orthogonal_standalone_contribution::<f32>();
}

#[test]
fn nested_subgrid_standalone_contribution_keeps_physical_axis_across_orthogonal_modes_f64() {
    qualify_orthogonal_standalone_contribution::<f64>();
}

struct OppositeTrackMeasuredGrid<S: LayoutScalar> {
    inputs: [LayoutInputOf<S>; 2],
    writing_mode: WritingMode,
    observations: std::cell::RefCell<Vec<(LeafMeasureInputOf<S>, Size<S>)>>,
}

impl<S: LayoutScalar> Traverse for OppositeTrackMeasuredGrid<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::option::IntoIter<usize>
    where
        Self: 'a;

    fn children(&self, node: usize) -> Self::Children<'_> {
        (node == 0).then_some(1).into_iter()
    }
    fn child_count(&self, node: usize) -> usize {
        usize::from(node == 0)
    }
    fn child(&self, node: usize, index: usize) -> usize {
        assert_eq!((node, index), (0, 0));
        1
    }
}

impl<S: LayoutScalar> LayoutTree for OppositeTrackMeasuredGrid<S> {
    type MeasureError = std::convert::Infallible;
    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap()
    }
    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }
    fn has_leaf_measurement(&self, node: usize) -> bool {
        node == 1
    }

    fn measure_leaf(
        &self,
        node: usize,
        request: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if node != 1 {
            return None;
        }
        let known = request.known_content_size();
        let available = request.available_content_size();
        let (known_inline, known_block, available_inline, available_block) = match self.writing_mode
        {
            WritingMode::HorizontalTb => {
                (known.width, known.height, available.width, available.height)
            }
            _ => (known.height, known.width, available.height, available.width),
        };
        // A declared host-content measurement, determined only by the real
        // public request. Unknown MIN inline is always 10. Unknown MAX inline
        // is 40 with a 20 block constraint and 80 when unconstrained. Known
        // content dimensions have priority; definite inline space uses the
        // declared min/max shrink-to-fit interval. No phase/call-count input
        // or fake compute result influences the response.
        let opposite =
            known_block.or_else(|| available_block.definite_value().map(|value| value.get()));
        let natural = S::from_f64(if opposite == Some(S::from_f64(20.0)) {
            40.0
        } else {
            80.0
        });
        let inline = known_inline.unwrap_or_else(|| match available_inline {
            MeasurementAvailableOf::MinContent => S::from_f64(10.0),
            MeasurementAvailableOf::MaxContent => natural,
            MeasurementAvailableOf::Definite(value) => {
                value.get().min(natural).max(S::from_f64(10.0))
            }
        });
        let measured = physical_size(
            self.writing_mode,
            inline,
            known_block.unwrap_or(S::from_f64(20.0)),
        );
        self.observations.borrow_mut().push((request, measured));
        Some(Ok(measured))
    }
}

fn qualify_first_axis_opposite_estimate<S: LayoutScalar>() {
    let mut failures = Vec::new();
    for writing_mode in [WritingMode::HorizontalTb, WritingMode::VerticalRl] {
        for fixed_opposite in [true, false] {
            let tree = OppositeTrackMeasuredGrid::<S> {
                inputs: [
                    LayoutInputOf::box_input(NodeInputOf {
                        display: Display::Grid,
                        writing_mode,
                        size: physical_size(
                            writing_mode,
                            PreferredSizeOf::MAX_CONTENT,
                            PreferredSizeOf::AUTO,
                        ),
                        grid_template_columns: vec![TrackComponentOf::AUTO],
                        grid_template_rows: vec![if fixed_opposite {
                            track(20.0)
                        } else {
                            TrackComponentOf::AUTO
                        }],
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    }),
                    LayoutInputOf::box_input(NodeInputOf {
                        display: Display::Block,
                        writing_mode,
                        item_is_replaced: true,
                        justify_self: Some(AlignItems::Start),
                        align_self: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    }),
                ],
                writing_mode,
                observations: std::cell::RefCell::new(Vec::new()),
            };
            // Grid2 §12.1 first-axis sizing estimates a definite 20 opposite
            // max track as 20; an AUTO opposite track is instead unbounded.
            // Therefore the initial inline MAX is 40 vs 80. The provider's
            // MIN remains 10 at either basis, so the later MIN-change condition
            // does not permit a column revisit to replace the AUTO control.
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
            )
            .unwrap();
            let output = |node| {
                batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == node)
                    .unwrap()
                    .output()
            };
            let actual = (output(0).size, output(1).size);
            let expected = (
                physical_size(
                    writing_mode,
                    S::from_f64(if fixed_opposite { 40.0 } else { 80.0 }),
                    S::from_f64(20.0),
                ),
                physical_size(writing_mode, S::from_f64(40.0), S::from_f64(20.0)),
            );
            // Requests/returned measurements are supplementary diagnostics;
            // the oracle is observable geometry over the declared content.
            eprintln!(
                "{} opposite_estimate {writing_mode:?} fixed_opposite={fixed_opposite}: actual={actual:?}, expected={expected:?}, measured={:?}",
                std::any::type_name::<S>(),
                tree.observations.borrow()
            );
            if actual != expected {
                failures.push(format!("{writing_mode:?} fixed_opposite={fixed_opposite}: actual={actual:?}, expected={expected:?}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn grid_first_axis_intrinsic_contribution_uses_definite_opposite_track_estimate_f32() {
    qualify_first_axis_opposite_estimate::<f32>();
}

#[test]
fn grid_first_axis_intrinsic_contribution_uses_definite_opposite_track_estimate_f64() {
    qualify_first_axis_opposite_estimate::<f64>();
}
