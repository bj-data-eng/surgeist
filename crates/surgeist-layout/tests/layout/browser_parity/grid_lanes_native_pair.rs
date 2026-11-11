//! Native comparison of separately authenticated Grid Lanes axis captures.

use std::collections::{BTreeMap, BTreeSet};

use super::capture_authority::SourceEquivalentMatching;
use super::{
    Error, Golden, Node, NodeKind, NumericComparisonSink, RootContext, Scalar, TestTree,
    compare_captured_with_batch, layout, root_context::QualifiedRootContext,
};
use layout::{Direction, GridAxisKind, NodeOutput, Point, Size, WritingMode};

const BOX_PATHS: [&str; 7] = ["/", "/0", "/0/0", "/0/1", "/1", "/1/0", "/1/1"];
const ITEM_SIZES: [Size; 2] = [Size::new(30.0, 50.0), Size::new(40.0, 60.0)];

#[derive(Clone, Copy)]
enum Templates {
    Both,
    GridAxisOnly,
}

#[derive(Clone, Copy)]
struct Expected {
    parent_mode: WritingMode,
    child_mode: WritingMode,
    direction: Direction,
}

#[derive(Clone, Copy)]
struct BoxPackets {
    unrounded: NodeOutput,
    final_output: NodeOutput,
}

type Packets = BTreeMap<&'static str, BoxPackets>;

pub(crate) fn verify_declared_pair(
    pair: &SourceEquivalentMatching,
    variant: &str,
    original_source: &[u8],
    control_source: &[u8],
    original: &Golden,
    control: &Golden,
) -> Result<(), Error> {
    let family = pair
        .original
        .strip_prefix("html/grid-lanes/grid_lanes_axes_")
        .and_then(|path| path.strip_suffix(".html"))
        .ok_or_else(|| Error::new("unknown original Grid Lanes source"))?;
    use WritingMode::{HorizontalTb, SidewaysLr, SidewaysRl, VerticalLr, VerticalRl};
    let (parent, child) = match family {
        "horizontal_tb_parallel" => (HorizontalTb, HorizontalTb),
        "horizontal_parent_orthogonal_child" => (HorizontalTb, VerticalRl),
        "sideways_lr_parallel" => (SidewaysLr, SidewaysLr),
        "sideways_opposing" => (SidewaysRl, SidewaysLr),
        "sideways_rl_parallel" => (SidewaysRl, SidewaysRl),
        "vertical_lr_parallel" => (VerticalLr, VerticalLr),
        "vertical_opposing" => (VerticalRl, VerticalLr),
        "vertical_parent_orthogonal_child" => (VerticalRl, HorizontalTb),
        "vertical_rl_parallel" => (VerticalRl, VerticalRl),
        _ => return Err(Error::new("unknown Grid Lanes source family")),
    };
    let direction = match variant {
        "border_box_ltr" | "content_box_ltr" => Direction::Ltr,
        "border_box_rtl" | "content_box_rtl" => Direction::Rtl,
        _ => return Err(Error::new("unknown Grid Lanes source variant")),
    };
    if original.name != format!("grid_lanes_axes_{family}__{variant}")
        || control.name != format!("grid_lanes_axes_{family}_control__{variant}")
    {
        return Err(Error::new(
            "Grid Lanes capture pair source or variant differs",
        ));
    }
    super::grid_lanes_source_delta::verify(original_source, control_source)
        .map_err(|error| Error::new(format!("{}: {error}", pair.original)))?;
    verify(original, control, parent, child, direction)
}

fn verify(
    original: &Golden,
    control: &Golden,
    parent_mode: WritingMode,
    child_mode: WritingMode,
    direction: Direction,
) -> Result<(), Error> {
    if !matches!(
        original.viewport.root_context,
        RootContext::Qualified(QualifiedRootContext::AbsoluteRootLocal { .. })
    ) || original.viewport.root_context != control.viewport.root_context
    {
        return Err(Error::new(
            "Grid Lanes pair root context operands differ or are unqualified",
        ));
    }
    if !original.use_rounding || !control.use_rounding {
        return Err(Error::new(
            "Grid Lanes pair requires both rounded capture variants",
        ));
    }
    let expected = Expected {
        parent_mode,
        child_mode,
        direction,
    };
    verify_source(original, Templates::Both, expected)?;
    verify_source(control, Templates::GridAxisOnly, expected)?;
    let original_packets = capture_packets(original, expected)?;
    let control_packets = capture_packets(control, expected)?;
    compare_packets(&original_packets, &control_packets)?;
    verify_physical_geometry(&original_packets, expected)
}

fn verify_source(golden: &Golden, templates: Templates, expected: Expected) -> Result<(), Error> {
    let root = &golden.root;
    verify_source_box(root, "/", WritingMode::HorizontalTb, expected.direction, 2)?;
    if root.style.get("display") != Some("block") || root.style.get("position") != Some("absolute")
    {
        return Err(Error::new(
            "Grid Lanes pair requires its absolute Block wrapper",
        ));
    }
    let (width, height) = if expected.parent_mode == WritingMode::HorizontalTb {
        ("70px", "110px")
    } else {
        ("110px", "70px")
    };
    for (index, lane) in root.children.iter().enumerate() {
        let path = format!("/{index}");
        verify_source_box(lane, &path, expected.parent_mode, expected.direction, 2)?;
        let column_flow = index == 0;
        let both = matches!(templates, Templates::Both);
        let columns = (both || !column_flow).then_some("30px 40px");
        let rows = (both || column_flow).then_some("50px 60px");
        let (flow, start, end) = if column_flow {
            ("column", "grid-row-start", "grid-row-end")
        } else {
            ("row", "grid-column-start", "grid-column-end")
        };
        if lane.style.get("display") != Some("grid-lanes")
            || lane.style.get("grid-auto-flow") != Some(flow)
            || lane.style.get("grid-template-columns") != columns
            || lane.style.get("grid-template-rows") != rows
            || lane.style.get("width") != Some(width)
            || lane.style.get("height") != Some(height)
        {
            return Err(Error::new(format!(
                "{} {path}: Grid Lanes flow, templates or physical dimensions changed",
                golden.name
            )));
        }
        for (item, (child, (width, height, first, last))) in lane
            .children
            .iter()
            .zip([("30px", "50px", "1", "2"), ("40px", "60px", "2", "3")])
            .enumerate()
        {
            let path = format!("{path}/{item}");
            verify_source_box(child, &path, expected.child_mode, expected.direction, 0)?;
            if child.style.get("width") != Some(width)
                || child.style.get("height") != Some(height)
                || child.style.get(start) != Some(first)
                || child.style.get(end) != Some(last)
            {
                return Err(Error::new(format!(
                    "{} {path}: authored physical size or definite lane placement changed",
                    golden.name
                )));
            }
        }
    }
    Ok(())
}

fn verify_source_box(
    node: &Node,
    path: &str,
    mode: WritingMode,
    direction: Direction,
    children: usize,
) -> Result<(), Error> {
    if node.kind != NodeKind::Div
        || node.text.is_some()
        || node.children.len() != children
        || super::parse_writing_mode(node.style.get("writing-mode"))? != mode
        || node.style.get("direction")
            != Some(match direction {
                Direction::Ltr => "ltr",
                Direction::Rtl => "rtl",
            })
    {
        return Err(Error::new(format!(
            "{path}: Grid Lanes box topology, writing mode or variant direction changed"
        )));
    }
    Ok(())
}

fn capture_packets(golden: &Golden, expected: Expected) -> Result<Packets, Error> {
    let mut packets = None;
    // The original's browser differences are diagnostic data. Both captures
    // still use their own admitted inputs and the complete comparison traversal.
    let mut numeric = NumericComparisonSink::CollectNumeric(Vec::new());
    let comparison = compare_captured_with_batch(golden, &mut numeric, |tree, batch| {
        packets = Some(snapshot(tree, batch, expected)?);
        Ok(())
    })?;
    if !comparison.endpoint_unobservable_fields().is_empty() {
        return Err(Error::new(
            "Grid Lanes pair has endpoint-unobservable geometry",
        ));
    }
    packets.ok_or_else(|| Error::new("Grid Lanes pair did not complete a Native batch"))
}

fn snapshot(
    tree: &TestTree,
    batch: &layout::CompletedLayoutBatch<usize>,
    expected: Expected,
) -> Result<Packets, Error> {
    let invalid = || Error::new("Grid Lanes pair requires exactly seven ordinary single boxes");
    if tree.nodes.len() != BOX_PATHS.len()
        || batch.node_layouts().len() != BOX_PATHS.len()
        || batch.root() != 0
        || !tree.paragraphs.is_empty()
        || !batch.placed_text().is_empty()
        || !batch.unrounded_inline_fragments().is_empty()
        || !batch.final_inline_fragments().is_empty()
        || batch.native_inline_box_families().next().is_some()
        || batch.native_inline_box_pieces().next().is_some()
    {
        return Err(invalid());
    }
    let [first, second] = tree.nodes[0].children[..] else {
        return Err(invalid());
    };
    let [first_a, first_b] = tree.nodes[first].children[..] else {
        return Err(invalid());
    };
    let [second_a, second_b] = tree.nodes[second].children[..] else {
        return Err(invalid());
    };
    let nodes = [0, first, first_a, first_b, second, second_a, second_b];
    if nodes.into_iter().collect::<BTreeSet<_>>().len() != BOX_PATHS.len() {
        return Err(invalid());
    }
    for (lane, axis, children) in [
        (first, GridAxisKind::Row, [first_a, first_b]),
        (second, GridAxisKind::Column, [second_a, second_b]),
    ] {
        let input = tree.nodes[lane].layout_input.as_box().ok_or_else(invalid)?;
        if input.grid_lanes_flow != layout::GridLanesFlow::new(axis, false, false) {
            return Err(Error::new(format!(
                "Native Grid Lanes flow must select {axis:?} tracks"
            )));
        }
        for (child, (start, end)) in children.into_iter().zip([(1, 2), (2, 3)]) {
            let input = tree.nodes[child]
                .layout_input
                .as_box()
                .ok_or_else(invalid)?;
            let placement = layout::RawGridPlacement::lines(start, end);
            let (grid, stacking) = match axis {
                GridAxisKind::Row => (&input.raw_grid_row, &input.raw_grid_column),
                GridAxisKind::Column => (&input.raw_grid_column, &input.raw_grid_row),
            };
            if grid != &placement || stacking != &layout::RawGridPlacement::AUTO {
                return Err(Error::new(format!(
                    "Native Grid Lanes item must occupy {axis:?} lines {start} / {end}"
                )));
            }
        }
    }
    let roles = [
        (layout::Display::Block, WritingMode::HorizontalTb, 0),
        (layout::Display::GridLanes, expected.parent_mode, 0),
        (layout::Display::Flex, expected.child_mode, 0),
        (layout::Display::Flex, expected.child_mode, 1),
        (layout::Display::GridLanes, expected.parent_mode, 1),
        (layout::Display::Flex, expected.child_mode, 0),
        (layout::Display::Flex, expected.child_mode, 1),
    ];
    BOX_PATHS
        .into_iter()
        .zip(nodes)
        .zip(roles)
        .map(|((path, node), (display, writing_mode, source_index))| {
            let input = tree.nodes[node].layout_input.as_box().ok_or_else(invalid)?;
            let position = if path == "/" {
                layout::Position::Absolute
            } else {
                layout::Position::Relative
            };
            if input.display != display
                || input.writing_mode != writing_mode
                || input.direction != expected.direction
                || input.position != position
                || tree.nodes[node].synthetic
            {
                return Err(Error::new(format!(
                    "{path}: lowered box role or axes changed"
                )));
            }
            let output = batch
                .node_layout(node)
                .and_then(layout::NodeLayoutRefOf::single_box)
                .ok_or_else(invalid)?;
            let unrounded = output.unrounded_output();
            let final_output = output.final_output();
            if unrounded.source_index.get() != source_index
                || final_output.source_index.get() != source_index
            {
                return Err(Error::new(format!(
                    "{path}: Native source sibling index changed"
                )));
            }
            Ok((
                path,
                BoxPackets {
                    unrounded,
                    final_output,
                },
            ))
        })
        .collect()
}

fn finite(packet: NodeOutput) -> bool {
    // Named baseline coordinates and canonical scroll geometry already enforce
    // finite construction. Check the remaining unconstrained numeric fields.
    let layout::NodeOutputOf {
        named_baselines: _,
        source_index: _,
        location,
        size,
        content_size,
        scroll_geometry: _,
        border,
        padding,
        margin,
    } = packet;
    [
        location.x,
        location.y,
        size.width,
        size.height,
        content_size.width,
        content_size.height,
    ]
    .into_iter()
    .chain(
        [border, padding, margin]
            .into_iter()
            .flat_map(|edges| [edges.top, edges.right, edges.bottom, edges.left]),
    )
    .all(Scalar::is_finite)
}

fn compare_packets(original: &Packets, control: &Packets) -> Result<(), Error> {
    for path in BOX_PATHS {
        let a = original
            .get(path)
            .ok_or_else(|| Error::new(format!("{path}: missing original box")))?;
        let b = control
            .get(path)
            .ok_or_else(|| Error::new(format!("{path}: missing control box")))?;
        for (phase, a, b) in [
            ("unrounded", a.unrounded, b.unrounded),
            ("final", a.final_output, b.final_output),
        ] {
            if !finite(a) || !finite(b) {
                return Err(Error::new(format!(
                    "{path}: nonfinite {phase} Native packet"
                )));
            }
            if a != b {
                return Err(Error::new(format!(
                    "{path}: {phase} Native packets differ (original {a:?}, control {b:?})"
                )));
            }
        }
    }
    Ok(())
}

fn verify_physical_geometry(packets: &Packets, expected: Expected) -> Result<(), Error> {
    let parent = if expected.parent_mode == WritingMode::HorizontalTb {
        Size::new(70.0, 110.0)
    } else {
        Size::new(110.0, 70.0)
    };
    let origins = item_origins(expected.parent_mode, expected.direction);
    for ((path, children), starts) in [("/0", ["/0/0", "/0/1"]), ("/1", ["/1/0", "/1/1"])]
        .into_iter()
        .zip(origins)
    {
        verify_box_geometry(path, packets[path], parent, None)?;
        for ((path, size), (x, y)) in children.into_iter().zip(ITEM_SIZES).zip(starts) {
            verify_box_geometry(path, packets[path], size, Some(Point::new(x, y)))?;
        }
    }
    Ok(())
}

fn verify_box_geometry(
    path: &str,
    packets: BoxPackets,
    size: Size,
    origin: Option<Point>,
) -> Result<(), Error> {
    for (phase, packet) in [
        ("unrounded", packets.unrounded),
        ("final", packets.final_output),
    ] {
        if packet.size != size || origin.is_some_and(|origin| packet.location != origin) {
            return Err(Error::new(format!(
                "{path}: {phase} Native geometry expected {origin:?} / {size:?}, got {:?} / {:?}",
                packet.location, packet.size
            )));
        }
    }
    Ok(())
}

fn item_origins(mode: WritingMode, direction: Direction) -> [[(Scalar, Scalar); 2]; 2] {
    use Direction::{Ltr, Rtl};
    use WritingMode::{HorizontalTb, SidewaysLr, SidewaysRl, VerticalLr, VerticalRl};
    // #997/#993 select rows for column flow and columns for row flow.
    // Writing Modes 4 Table 11 projects the fixed track starts and each
    // item's zero stacking offset onto its parent's physical start edges.
    match (mode, direction) {
        (HorizontalTb, Ltr) => [[(0.0, 0.0), (0.0, 50.0)], [(0.0, 0.0), (30.0, 0.0)]],
        (HorizontalTb, Rtl) => [[(40.0, 0.0), (30.0, 50.0)], [(40.0, 0.0), (0.0, 0.0)]],
        (VerticalRl | SidewaysRl, Ltr) => [[(80.0, 0.0), (20.0, 0.0)], [(80.0, 0.0), (70.0, 30.0)]],
        (VerticalRl | SidewaysRl, Rtl) => {
            [[(80.0, 20.0), (20.0, 10.0)], [(80.0, 20.0), (70.0, -20.0)]]
        }
        (VerticalLr, Ltr) | (SidewaysLr, Rtl) => {
            [[(0.0, 0.0), (50.0, 0.0)], [(0.0, 0.0), (0.0, 30.0)]]
        }
        (VerticalLr, Rtl) | (SidewaysLr, Ltr) => {
            [[(0.0, 20.0), (50.0, 10.0)], [(0.0, 20.0), (0.0, -20.0)]]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture_variants::fixture_cases;

    #[test]
    fn retained_vertical_rl_pair_requires_its_variant_and_shared_inputs() {
        use super::super::capture_authority::{
            OriginalDiagnosticReceipt, Qualification, verify_grid_lanes_shared_inputs,
        };

        let original_receipt = OriginalDiagnosticReceipt::parse(include_str!(
            "captured/grid-lanes-axis-original-diagnostics/diagnostics.json"
        ))
        .unwrap();
        let mut controls = Qualification::parse(include_str!(
            "captured/grid-lanes-axis-controls/qualification.json"
        ))
        .unwrap();
        let record = controls
            .records
            .iter()
            .find(|record| {
                record.source
                    == "html/qualification/grid_lanes_axes_vertical_rl_parallel_control.html"
            })
            .unwrap();
        let association =
            serde_json::from_value(record.value["matching_publication"].clone()).unwrap();
        let original_source = include_bytes!(
            "captured/grid-lanes-axis-original-diagnostics/html/grid-lanes/grid_lanes_axes_vertical_rl_parallel.html"
        );
        let control_source = include_bytes!(
            "captured/grid-lanes-axis-controls/html/qualification/grid_lanes_axes_vertical_rl_parallel_control.html"
        );
        let original = Golden::parse(include_str!("captured/grid-lanes-axis-original-diagnostics/xml/grid-lanes/grid_lanes_axes_vertical_rl_parallel__border_box_rtl.xml")).unwrap();
        let control = Golden::parse(include_str!("captured/grid-lanes-axis-controls/xml/qualification/grid_lanes_axes_vertical_rl_parallel_control__border_box_rtl.xml")).unwrap();
        for (variant, _) in fixture_cases() {
            let result = verify_declared_pair(
                &association,
                variant,
                original_source,
                control_source,
                &original,
                &control,
            );
            assert_eq!(
                result.is_ok(),
                variant == "border_box_rtl",
                "{variant}: {result:?}"
            );
        }

        verify_grid_lanes_shared_inputs(
            &original_receipt.retained_inputs,
            &controls.retained_inputs,
        )
        .unwrap();
        let source_provenance = "origin/source-provenance.json";
        let other = controls.retained_inputs["origin/source-receipt.json"].clone();
        assert_ne!(controls.retained_inputs[source_provenance], other);
        controls
            .retained_inputs
            .insert(source_provenance.into(), other);
        let error = verify_grid_lanes_shared_inputs(
            &original_receipt.retained_inputs,
            &controls.retained_inputs,
        )
        .unwrap_err();
        assert!(error.to_string().contains(source_provenance), "{error}");
    }

    const HORIZONTAL: Expected = Expected {
        parent_mode: WritingMode::HorizontalTb,
        child_mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
    };

    fn fixture(templates: Templates) -> Golden {
        let (columns, rows) = match templates {
            Templates::Both => (
                r#" grid-template-columns="30px 40px""#,
                r#" grid-template-rows="50px 60px""#,
            ),
            Templates::GridAxisOnly => ("", ""),
        };
        // Test-owned V4 inputs and observations for a horizontal pair.
        Golden::parse(&format!(
            r#"<test name="lanes-pair" measurement-version="4" observation-frame="root-border-local" use-rounding="true">
  <viewport root-context="absolute-root-local" containing-width="800px" containing-height="600px"
    containing-direction="ltr" hypothetical-direction="ltr" hypothetical-display="block"
    hypothetical-anchor="inline-start" hypothetical-inline-offset="0px" fallback-icb-width="800px"
    fallback-icb-height="600px" fallback-nearest-scrollport="none"
    fallback-containing-width-limit="unbounded" fallback-containing-height-limit="unbounded"/>
  <input>
    <div source-tag="div" display="block" direction="ltr" position="absolute" grid-alignment-profile='{{"profile":1,"owners":[]}}'>
      <div source-tag="div" display="grid-lanes" direction="ltr" width="70px" height="110px"
        grid-auto-flow="column" grid-template-rows="50px 60px"{columns}>
        <div source-tag="div" display="flex" direction="ltr" width="30px" height="50px" grid-row-start="1" grid-row-end="2"/>
        <div source-tag="div" display="flex" direction="ltr" width="40px" height="60px" grid-row-start="2" grid-row-end="3"/>
      </div>
      <div source-tag="div" display="grid-lanes" direction="ltr" width="70px" height="110px"
        grid-auto-flow="row" grid-template-columns="30px 40px"{rows}>
        <div source-tag="div" display="flex" direction="ltr" width="30px" height="50px" grid-column-start="1" grid-column-end="2"/>
        <div source-tag="div" display="flex" direction="ltr" width="40px" height="60px" grid-column-start="2" grid-column-end="3"/>
      </div>
    </div>
  </input>
  <expectations>
    <node x="0" y="0" width="70" height="220">
      <node x="0" y="0" width="70" height="110">
        <node x="0" y="0" width="30" height="50"/>
        <node x="0" y="50" width="40" height="60"/>
      </node>
      <node x="0" y="110" width="70" height="110">
        <node x="0" y="0" width="30" height="50"/>
        <node x="30" y="0" width="40" height="60"/>
      </node>
    </node>
  </expectations>
  <diagnostics>
    <world-rounded-layout node-path="0" x="0" y="0" width="70" height="220"/>
    <world-rounded-layout node-path="0.0" x="0" y="0" width="70" height="110"/>
    <world-rounded-layout node-path="0.0.0" x="0" y="0" width="30" height="50"/>
    <world-rounded-layout node-path="0.0.1" x="0" y="50" width="40" height="60"/>
    <world-rounded-layout node-path="0.1" x="0" y="110" width="70" height="110"/>
    <world-rounded-layout node-path="0.1.0" x="0" y="0" width="30" height="50"/>
    <world-rounded-layout node-path="0.1.1" x="30" y="0" width="40" height="60"/>
  </diagnostics>
</test>"#
        ))
        .unwrap()
    }

    fn verify_horizontal(original: &Golden, control: &Golden) -> Result<(), Error> {
        verify(
            original,
            control,
            HORIZONTAL.parent_mode,
            HORIZONTAL.child_mode,
            HORIZONTAL.direction,
        )
    }

    #[test]
    fn native_pair_accepts_equal_packets_with_an_original_diagnostic_difference() {
        let mut original = fixture(Templates::Both);
        original.expectations.children[0].children[1].x = Some(7.0);
        let control = fixture(Templates::GridAxisOnly);
        verify_horizontal(&original, &control).unwrap();
    }

    #[test]
    fn native_pair_refuses_changed_context_or_rounding() {
        let original = fixture(Templates::Both);
        let mut control = fixture(Templates::GridAxisOnly);
        control.use_rounding = false;
        assert!(
            verify_horizontal(&original, &control)
                .unwrap_err()
                .to_string()
                .contains("rounded capture variants")
        );
        control.use_rounding = true;
        let RootContext::Qualified(QualifiedRootContext::AbsoluteRootLocal {
            hypothetical_inline_offset,
            ..
        }) = &mut control.viewport.root_context
        else {
            panic!("absolute-local fixture");
        };
        *hypothetical_inline_offset = 1.0;
        assert!(
            verify_horizontal(&original, &control)
                .unwrap_err()
                .to_string()
                .contains("root context operands differ")
        );
    }

    #[test]
    fn native_pair_refuses_direction_that_disagrees_with_the_selected_variant() {
        let original = fixture(Templates::Both);
        let control = fixture(Templates::GridAxisOnly);
        assert!(
            verify(
                &original,
                &control,
                HORIZONTAL.parent_mode,
                HORIZONTAL.child_mode,
                Direction::Rtl,
            )
            .unwrap_err()
            .to_string()
            .contains("variant direction changed")
        );
    }

    #[test]
    fn native_pair_refuses_sub_rounding_drift_with_equal_final_packets() {
        let original = capture_packets(&fixture(Templates::Both), HORIZONTAL).unwrap();
        let mut control = original.clone();
        let item = control.get_mut("/0/1").unwrap();
        item.unrounded.location.x += 0.03125;
        assert_eq!(original["/0/1"].final_output, item.final_output);
        assert!(
            compare_packets(&original, &control)
                .unwrap_err()
                .to_string()
                .starts_with("/0/1: unrounded Native packets differ")
        );
    }

    #[test]
    fn native_pair_refuses_equal_nonfinite_packets() {
        let mut packets = capture_packets(&fixture(Templates::Both), HORIZONTAL).unwrap();
        packets.get_mut("/1").unwrap().unrounded.content_size.height = Scalar::INFINITY;
        assert_eq!(
            compare_packets(&packets, &packets).unwrap_err().to_string(),
            "/1: nonfinite unrounded Native packet"
        );
    }
}
