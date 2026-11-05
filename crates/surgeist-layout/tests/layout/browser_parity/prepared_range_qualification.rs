//! Exact comparison of the prepared continuity fixture under the Native source-anchor policy.

use super::*;
use exact_differences::{QualifiedComparison, QualifiedVerdict, RawVerdict};
use serde::Serialize;
use surgeist_text::Range;

pub const SOURCE: &str = "html/block/prepared_text_continuity_bidi.html";
pub const POLICY_SOURCE: &str = "docs/text-layout-contract.md";
pub(crate) const POLICY_ANCHOR: &str = "nonempty-text-range-fragments";
const POLICY_TITLE: &str = "Nonempty Text range fragments";
pub(crate) const RETAINED_POLICY: &str = "semantic/prepared-range-policy.md";
pub(crate) const POLICY_SHA256: &str =
    "ef6219d28d15d9131a45d0c48edef12436333a1e1f2868f54d9ec8847d09ada0";

pub struct PolicyIdentity(&'static str);

impl PolicyIdentity {
    pub fn from_contract(contract: &str) -> Result<Self, Error> {
        Self::verified_section(contract)?;
        Ok(Self(POLICY_SHA256))
    }

    pub fn verified_section(contract: &str) -> Result<&str, Error> {
        let mut offset = 0;
        let mut headings = contract.split_inclusive('\n').filter_map(|line| {
            offset += line.len();
            is_policy_heading(line).then_some(offset)
        });
        let start = headings
            .next()
            .ok_or_else(|| Error::new("missing prepared Range policy section"))?;
        if headings.next().is_some() {
            return Err(Error::new("duplicate prepared Range policy section"));
        }
        let section = &contract[start..];
        let length = section
            .split_inclusive('\n')
            .take_while(|line| {
                !["# ", "## ", "### "]
                    .iter()
                    .any(|prefix| line.starts_with(prefix))
            })
            .map(str::len)
            .sum();
        let section = &section[..length];
        if capture_authority::sha256(section.as_bytes()) != POLICY_SHA256 {
            return Err(Error::new("prepared Range policy section changed"));
        }
        Ok(section)
    }
}
fn is_policy_heading(line: &str) -> bool {
    let line = line.trim_end();
    let heading = line.trim_start_matches(' ');
    let marker_len = heading.bytes().take_while(|byte| *byte == b'#').count();
    if line.len() - heading.len() > 3 || !(1..=6).contains(&marker_len) {
        return false;
    }
    let title = &heading[marker_len..];
    if !title.starts_with([' ', '\t']) {
        return false;
    }
    let title = title.trim_start_matches([' ', '\t']);
    let before_closing_marker = title.trim_end_matches('#');
    let title = if before_closing_marker.ends_with([' ', '\t']) {
        before_closing_marker.trim_end_matches([' ', '\t'])
    } else {
        title
    };
    title.eq_ignore_ascii_case(POLICY_TITLE)
}

const FONT_SIZE: f64 = 16.0;
const LINE_HEIGHT: f64 = 20.0;
// The font has ascent 800 at UPEM 1000. The remaining leading is split
// around the font band; Layout places the baseline on the physical pixel grid.
const NATURAL_BASELINE: f64 = (LINE_HEIGHT - FONT_SIZE) / 2.0 + FONT_SIZE * 0.8;
const FIRST_BASELINE: f64 = 15.0;
const BLOCK_SHIFT: f64 = FIRST_BASELINE - NATURAL_BASELINE;

#[derive(Debug, PartialEq, Serialize)]
struct BoxComparison {
    path: String,
    field: &'static str,
    browser: Scalar,
    native: Scalar,
}

#[derive(Debug, PartialEq, Serialize)]
struct RangeComparison {
    path: String,
    row: usize,
    occurrence: u64,
    line: usize,
    edge: &'static str,
    native_source: [usize; 2],
    native_pieces: Vec<RangePieceKey>,
    browser_start: Scalar,
    native_start: Scalar,
    browser_advance: Scalar,
    native_advance: Scalar,
}

#[derive(Serialize)]
pub(super) struct Comparisons {
    case: String,
    boxes: Vec<BoxComparison>,
    ranges: Vec<RangeComparison>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub(super) struct RangePieceKey {
    pub(super) unit: usize,
    pub(super) piece: usize,
    pub(super) local: [usize; 2],
    pub(super) aggregate: [usize; 2],
}

struct RangeSelector {
    path: &'static str,
    row: usize,
    occurrence: u64,
    line: usize,
    source: [usize; 2],
    pieces: &'static [RangePieceKey],
    advance: Scalar,
}

// Source intervals use aggregate UTF-16 units.
const RANGE_SELECTORS: [RangeSelector; 5] = [
    RangeSelector {
        path: "/0",
        row: 0,
        occurrence: 0,
        line: 0,
        source: [0, 1],
        pieces: &[RangePieceKey {
            unit: 0,
            piece: 0,
            local: [0, 1],
            aggregate: [0, 1],
        }],
        advance: 16.0,
    },
    RangeSelector {
        path: "/1",
        row: 0,
        occurrence: 1,
        line: 0,
        source: [1, 2],
        pieces: &[RangePieceKey {
            unit: 0,
            piece: 0,
            local: [0, 1],
            aggregate: [1, 2],
        }],
        advance: 0.0,
    },
    RangeSelector {
        path: "/1",
        row: 1,
        occurrence: 1,
        line: 0,
        source: [2, 4],
        pieces: &[
            RangePieceKey {
                unit: 1,
                piece: 0,
                local: [1, 2],
                aggregate: [2, 3],
            },
            RangePieceKey {
                unit: 2,
                piece: 0,
                local: [2, 3],
                aggregate: [3, 4],
            },
        ],
        advance: 32.0,
    },
    RangeSelector {
        path: "/1",
        row: 2,
        occurrence: 1,
        line: 0,
        source: [4, 5],
        pieces: &[RangePieceKey {
            unit: 3,
            piece: 0,
            local: [3, 4],
            aggregate: [4, 5],
        }],
        advance: 0.0,
    },
    RangeSelector {
        path: "/2",
        row: 0,
        occurrence: 2,
        line: 1,
        source: [6, 7],
        pieces: &[RangePieceKey {
            unit: 0,
            piece: 0,
            local: [1, 2],
            aggregate: [6, 7],
        }],
        advance: 16.0,
    },
];

impl Comparisons {
    pub(super) fn row_pieces(
        &self,
        path: &str,
        browser: &[InlineRangeRectangleExpectation],
    ) -> Result<Vec<&'static [RangePieceKey]>, Error> {
        let owner = exact_differences::source_path_from_diagnostic(&self.case, path)?;
        let mut row_pieces = Vec::new();
        for RangeSelector {
            path: owner_path,
            row,
            occurrence,
            line,
            pieces,
            ..
        } in RANGE_SELECTORS
        {
            if owner_path != owner {
                continue;
            }
            let item = browser
                .get(row)
                .ok_or_else(|| Error::new(format!("{path}: missing prepared Range row {row}")))?;
            if item.source_segment_id != occurrence
                || item.line_index != line
                || item.physical_start_edge != PhysicalStartEdge::Left
            {
                return Err(Error::new(format!(
                    "{path}: prepared Range row {row} identity changed"
                )));
            }
            row_pieces.push(pieces);
        }
        if row_pieces.len() != browser.len() {
            return Err(Error::new(format!(
                "{path}: prepared Range row census changed"
            )));
        }
        Ok(row_pieces)
    }

    pub(super) fn record_box(
        &mut self,
        path: &str,
        field: NumericComparisonField,
        native: Scalar,
        browser: Scalar,
    ) -> Result<(), Error> {
        if !native.is_finite() || !browser.is_finite() {
            return Err(Error::new(format!("{path}: nonfinite box comparison")));
        }
        self.boxes.push(BoxComparison {
            path: exact_differences::source_path_from_diagnostic(&self.case, path)?,
            field: field.as_str(),
            browser,
            native,
        });
        Ok(())
    }

    pub(super) fn record_range(
        &mut self,
        path: &str,
        pair: RangeRectanglePair,
    ) -> Result<(), Error> {
        let edge = match pair.physical_start_edge {
            PhysicalStartEdge::Left => "left",
            PhysicalStartEdge::Right => "right",
            PhysicalStartEdge::Top => "top",
            PhysicalStartEdge::Bottom => "bottom",
        };
        self.ranges.push(RangeComparison {
            path: exact_differences::source_path_from_diagnostic(&self.case, path)?,
            row: pair.index,
            occurrence: pair.source_segment_id,
            line: pair.line_index,
            edge,
            native_source: [pair.native_source.start, pair.native_source.end],
            native_pieces: pair.native_pieces,
            browser_start: pair.browser_start,
            native_start: pair.native_start,
            browser_advance: pair.browser_advance,
            native_advance: pair.native_advance,
        });
        Ok(())
    }
}

pub fn qualify(
    golden: &Golden,
    source: &str,
    variant: &str,
    policy: &PolicyIdentity,
    mut raw: impl std::io::Write,
) -> Result<(), Error> {
    let result = compare(golden, source, variant)?;
    serde_json::to_writer(&mut raw, &result.observations).map_err(|e| Error::new(e.to_string()))?;
    writeln!(raw).map_err(|e| Error::new(e.to_string()))?;

    let (_, qualified) = result.into_outcomes()?;
    if matches!(qualified?.qualified, QualifiedVerdict::Intentional { .. }) {
        writeln!(
            raw,
            "{{\"case\":\"{}\",\"verdict\":\"intentional\",\"policy\":\"{POLICY_SOURCE}#{POLICY_ANCHOR}\",\"policy_sha256\":\"{}\"}}",
            golden.name, policy.0
        )
    } else {
        writeln!(
            raw,
            "{{\"case\":\"{}\",\"verdict\":\"matching-control\"}}",
            golden.name
        )
    }
    .map_err(|e| Error::new(e.to_string()))
}

pub(crate) struct PreparedComparison {
    observations: Comparisons,
    complete: bool,
    intentional: Result<bool, Error>,
}

fn incomplete() -> Error {
    Error::new("prepared Range capture requires complete observations")
}

impl PreparedComparison {
    pub(crate) fn into_outcomes(
        self,
    ) -> Result<(Vec<String>, Result<QualifiedComparison, Error>), Error> {
        let fields = self.raw_differences()?;
        let qualified = self.intentional.map(|intentional| {
            if intentional {
                QualifiedComparison {
                    raw: RawVerdict::Different {
                        fields: fields.len(),
                    },
                    qualified: QualifiedVerdict::Intentional {
                        policy: format!("{POLICY_SOURCE}#{POLICY_ANCHOR}"),
                        edition: POLICY_SHA256.to_owned(),
                    },
                }
            } else {
                QualifiedComparison {
                    raw: RawVerdict::Matches,
                    qualified: QualifiedVerdict::MatchingControl,
                }
            }
        });
        Ok((fields, qualified))
    }

    fn raw_differences(&self) -> Result<Vec<String>, Error> {
        if !self.complete {
            return Err(incomplete());
        }
        Ok(self
            .observations
            .boxes
            .iter()
            .filter(|field| field.browser != field.native)
            .map(|field| format!("{field:?}"))
            .chain(
                self.observations
                    .ranges
                    .iter()
                    .filter(|row| {
                        row.browser_start != row.native_start
                            || row.browser_advance != row.native_advance
                    })
                    .map(|row| format!("{row:?}")),
            )
            .collect())
    }
}

pub(crate) fn compare(
    golden: &Golden,
    source: &str,
    variant: &str,
) -> Result<PreparedComparison, Error> {
    let rtl = match variant {
        "border_box_ltr" | "content_box_ltr" => false,
        "border_box_rtl" | "content_box_rtl" => true,
        _ => return Err(Error::new("unregistered prepared Range variant")),
    };
    if source != SOURCE
        || golden.name != format!("prepared_text_continuity_bidi__{variant}")
        || !golden.has_prepared_range_frame()
        || golden.has_element_scroll_sizes()
        || golden.measured_leaf.is_some()
        || golden.has_flex_content_text()
    {
        return Err(Error::new("unregistered prepared Range capture authority"));
    }
    let mut custody = None;
    let mut sink = NumericComparisonSink::PreparedRanges(Comparisons {
        case: golden.name.clone(),
        boxes: Vec::new(),
        ranges: Vec::new(),
    });
    let comparison = compare_captured_with_batch(golden, &mut sink, |_, batch| {
        custody = Some(verify_native(golden, batch, rtl));
        Ok(())
    })?;
    let NumericComparisonSink::PreparedRanges(comparisons) = sink else {
        unreachable!("prepared Range collector remains selected");
    };
    let complete = comparison.endpoint_unobservable_fields().is_empty();
    let intentional = if complete {
        custody
            .ok_or_else(|| Error::new("missing completed Native batch"))
            .and_then(|native| native)
            .and_then(|()| verify_comparisons(&comparisons, rtl))
            .map(|()| rtl)
    } else {
        Err(incomplete())
    };
    Ok(PreparedComparison {
        observations: comparisons,
        complete,
        intentional,
    })
}

fn verify_comparisons(comparisons: &Comparisons, rtl: bool) -> Result<(), Error> {
    // Both sizing modes have zero box edges. Three 16px cells fit the
    // first 20px line; the fourth glyph follows the consumed break space.
    let box_fields = [("x", 0.0), ("y", 0.0), ("width", 72.0), ("height", 40.0)];
    let starts = if rtl {
        [24.0, 40.0, 40.0, 24.0, 56.0]
    } else {
        [0.0, 16.0, 16.0, 48.0, 0.0]
    };
    if comparisons.boxes.len() != box_fields.len()
        || comparisons.ranges.len() != RANGE_SELECTORS.len()
    {
        return Err(Error::new("prepared Range observation census changed"));
    }
    for (actual, (field, value)) in comparisons.boxes.iter().zip(box_fields) {
        if (
            actual.path.as_str(),
            actual.field,
            actual.browser,
            actual.native,
        ) != ("/", field, value, value)
        {
            return Err(Error::new(format!(
                "prepared Range box comparison changed: {actual:?}"
            )));
        }
    }
    for (
        index,
        (
            actual,
            RangeSelector {
                path,
                row,
                occurrence,
                line,
                source,
                pieces,
                advance,
            },
        ),
    ) in comparisons.ranges.iter().zip(RANGE_SELECTORS).enumerate()
    {
        let native_start = starts[index];
        let browser_start = if rtl && source == [4, 5] {
            72.0
        } else {
            native_start
        };
        let expected = RangeComparison {
            path: path.to_owned(),
            row,
            occurrence,
            line,
            edge: "left",
            native_source: source,
            native_pieces: pieces.to_vec(),
            browser_start,
            native_start,
            browser_advance: advance,
            native_advance: advance,
        };
        if *actual != expected {
            return Err(Error::new(format!(
                "prepared Range comparison changed: expected {expected:?}, observed {actual:?}"
            )));
        }
    }
    Ok(())
}

fn verify_native(
    golden: &Golden,
    batch: &layout::CompletedLayoutBatch<usize>,
    rtl: bool,
) -> Result<(), Error> {
    let [placed] = batch.placed_text() else {
        return Err(Error::new("expected one placed prepared paragraph"));
    };
    let text = placed.layout();
    verify_node_packets(batch, rtl)?;
    verify_selected_lines(text, rtl)?;
    let Some(pdi_origin) = golden
        .root
        .children
        .get(2)
        .and_then(|child| child.prepared_origin.as_ref())
    else {
        return Err(Error::new("missing original PDI occurrence"));
    };
    if placed.container() != 0
        || pdi_origin.view().source().text().code_units() != [0x2067, 88, 88, 0x2069]
        || text.analyzed_source().identity()
            != pdi_origin.prepared_text().analyzed_source().identity()
        || text
            .run_views()
            .any(|run| run.face().bytes() != pinned_font::BYTES || run.face().index() != 0)
    {
        return Err(Error::new(
            "prepared source, placement or effective face changed",
        ));
    }
    let pdi = Range::new(4, 5);
    let local = Range::new(3, 4);
    let query = text
        .source_range_fragments(pdi_origin, local)
        .map_err(|error| Error::new(format!("PDI Range refusal: {error:?}")))?;
    let mut units = query.units();
    let Some(unit) = units.next() else {
        return Err(Error::new("missing placed PDI Range unit"));
    };
    let mut intersections = unit.selected_intersections();
    let selected = intersections.next();
    let mut pieces = unit.pieces();
    let Some(surgeist_text::SourceRangeGeometry::Line(piece)) = pieces.next() else {
        return Err(Error::new("missing PDI line font band"));
    };
    let cursor = surgeist_text::Cursor::new(4, surgeist_text::Affinity::After);
    let x = if rtl { 24.0 } else { 48.0 };
    let instance = piece.effective_instance();
    let font_band = piece.rect();
    if units.next().is_some()
        || intersections.next().is_some()
        || selected.map(|intersection| (intersection.local_range(), intersection.aggregate_range()))
            != Some((local, pdi))
        || pieces.next().is_some()
        || unit.line_index() != 0
        || (
            font_band.origin.x,
            font_band.size.width,
            font_band.size.height,
        ) != (f64::from(x), 0.0, FONT_SIZE)
        || !metric_matches(font_band.origin.y, FIRST_BASELINE - FONT_SIZE * 0.8)
        || piece.baseline() != FIRST_BASELINE
        || instance.face().bytes() != pinned_font::BYTES
        || instance.face().index() != 0
        || instance.font_size() != FONT_SIZE as f32
        || instance.axis_count() != 0
        || instance.x_shear() != 0.0
        || instance.embolden() != 0.0
    {
        return Err(Error::new(
            "placed PDI Range source, line geometry or effective font changed",
        ));
    }
    let last_origin = golden
        .root
        .children
        .get(4)
        .and_then(|child| child.prepared_origin.as_ref())
        .ok_or_else(|| Error::new("missing second-line source occurrence"))?;
    let last_query = text
        .source_range_fragments(last_origin, Range::new(1, 2))
        .map_err(|error| Error::new(format!("second-line Range refusal: {error:?}")))?;
    let last = last_query
        .units()
        .next()
        .ok_or_else(|| Error::new("missing second-line Range unit"))?;
    if last.line_index() != 1 {
        return Err(Error::new("second-line source occurrence changed lines"));
    }
    let fragments = batch.unrounded_inline_fragments();
    verify_fragment_packets(
        golden,
        fragments,
        batch.final_inline_fragments(),
        &[unit.finalized_line(), last.finalized_line()],
    )?;
    let mut matching = fragments.iter().filter(|entry| {
        let view = entry.fragment().finalized_fragment().view();
        view.origin().as_ref() == Some(pdi_origin) && view.aggregate_range() == Some(pdi)
    });
    let Some(entry) = matching.next() else {
        return Err(Error::new("missing issued PDI fragment"));
    };
    let fragment = entry.fragment();
    let issued = fragment.finalized_fragment();
    let line = issued.finalized_line();
    if matching.next().is_some()
        || fragment.line_index() != 0
        || line != unit.finalized_line()
        || !unit.fragments().any(|actual| actual == *issued)
        || !line.fragments().any(|actual| actual == *issued)
        || fragment.baseline().y != FIRST_BASELINE as Scalar
        || fragment.rect().map(|rect| (rect.origin(), rect.size()))
            != Some((
                layout::Point::new(x, BLOCK_SHIFT as Scalar),
                layout::Size::new(0.0, 20.0),
            ))
    {
        return Err(Error::new(format!(
            "issued PDI fragment or selected line changed: line {}, rect {:?}, baseline {:?}",
            fragment.line_index(),
            fragment.rect(),
            fragment.baseline(),
        )));
    }
    let position = text
        .positions()
        .find(|position| position.source_cursor() == cursor)
        .ok_or_else(|| Error::new("missing issued PDI position"))?;
    let geometry = text
        .position_geometry(&position)
        .map_err(|error| Error::new(format!("PDI placement refusal: {error:?}")))?;
    let caret = geometry.rect();
    if (caret.origin().x, caret.size().width, caret.size().height)
        != (f64::from(x), 1.0, LINE_HEIGHT)
        || !metric_matches(caret.origin().y, BLOCK_SHIFT)
    {
        return Err(Error::new("placed PDI source anchor changed"));
    }
    Ok(())
}

fn verify_fragment_packets(
    golden: &Golden,
    fragments: &[layout::InlineFragmentOutputEntry<usize>],
    final_fragments: &[layout::InlineFragmentOutputEntry<usize>],
    lines: &[surgeist_text::FinalizedLine; 2],
) -> Result<(), Error> {
    // The captured paragraph has one canonical anchor per selected line.
    // Both packet channels must project the same issued Text fragment.
    if fragments != final_fragments
        || fragments.len()
            != lines
                .iter()
                .map(|line| line.fragments().count())
                .sum::<usize>()
    {
        return Err(Error::new(
            "prepared Range fragment packet census or phase changed",
        ));
    }
    for (line_index, line) in lines.iter().enumerate() {
        let baseline = FIRST_BASELINE + line_index as f64 * LINE_HEIGHT;
        for issued in line.fragments() {
            let view = issued.view();
            let child = golden
                .root
                .children
                .iter()
                .position(|child| match view.owner() {
                    surgeist_text::FinalizedFragmentOwner::Origin(origin) => {
                        child.prepared_origin.as_ref() == Some(&origin)
                    }
                    surgeist_text::FinalizedFragmentOwner::Control(control) => {
                        child.prepared_control.as_ref() == Some(&control)
                    }
                    _ => false,
                })
                .ok_or_else(|| Error::new("issued fragment has no captured source owner"))?;
            let mut matching = fragments
                .iter()
                .filter(|entry| entry.fragment().finalized_fragment() == &issued);
            let entry = matching
                .next()
                .ok_or_else(|| Error::new("missing issued source fragment packet"))?;
            let actual = entry.fragment();
            let expected = view.bounds().map(|bounds| {
                (
                    layout::Point::new(
                        bounds.origin.x as Scalar,
                        (baseline + (bounds.origin.y - line.feedback().baseline)) as Scalar,
                    ),
                    layout::Size::new(bounds.size.width as Scalar, bounds.size.height as Scalar),
                )
            });
            if matching.next().is_some()
                || entry.owner() != &layout::InlineFragmentOwner::Node(child + 1)
                || actual.line_index() != line_index
                || actual.baseline() != layout::Point::new(0.0, baseline as Scalar)
                || actual.rect().map(|rect| (rect.origin(), rect.size())) != expected
            {
                return Err(Error::new(format!(
                    "issued source fragment packet changed at source child {child} on line {line_index}"
                )));
            }
        }
    }
    Ok(())
}

fn verify_node_packets(
    batch: &layout::CompletedLayoutBatch<usize>,
    rtl: bool,
) -> Result<(), Error> {
    let direction = if rtl {
        layout::Direction::Rtl
    } else {
        layout::Direction::Ltr
    };
    let children = if rtl {
        [
            (24.0, 16.0, 20.0),
            (40.0, 0.0, 0.0),
            (24.0, 48.0, 20.0),
            (24.0, 0.0, 0.0),
            (24.0, 48.0, 40.0),
        ]
    } else {
        [
            (0.0, 16.0, 20.0),
            (16.0, 0.0, 0.0),
            (16.0, 32.0, 20.0),
            (48.0, 0.0, 0.0),
            (0.0, 48.0, 40.0),
        ]
    };
    let bounds = layout::ScrollRect::try_new(layout::Point::ZERO, layout::Size::new(72.0, 40.0))
        .map_err(|_| Error::new("invalid authored root bounds"))?;
    for (entries, content_height) in [
        (batch.unrounded_entries(), (40.0 + BLOCK_SHIFT) as Scalar),
        (batch.final_entries(), 40.0),
    ] {
        if entries.len() != children.len() + 1 {
            return Err(Error::new("prepared Range packet census changed"));
        }
        let root = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .ok_or_else(|| Error::new("missing prepared Range root packet"))?
            .output();
        let scroll = root
            .scroll_geometry
            .ok_or_else(|| Error::new("missing prepared Range root scroll geometry"))?;
        let expected_root = layout::NodeOutput {
            size: bounds.size(),
            content_size: layout::Size::new(72.0, content_height),
            ..layout::NodeOutput::new()
        };
        if (layout::NodeOutput {
            scroll_geometry: None,
            ..root
        }) != expected_root
            || scroll.flow_axes()
                != layout::FlowAxes::new(layout::WritingMode::HorizontalTb, direction)
            || (scroll.used_overflow_x(), scroll.used_overflow_y())
                != (layout::Overflow::Visible, layout::Overflow::Visible)
            || [
                scroll.border_box(),
                scroll.padding_box(),
                scroll.content_box(),
                scroll.scrollport(),
            ]
            .into_iter()
            .any(|rect| rect != bounds)
            || scroll.scrollable_overflow().origin() != layout::Point::ZERO
            || scroll.scrollable_overflow().size() != expected_root.content_size
        {
            return Err(Error::new(format!(
                "prepared Range root packet changed: {root:?}"
            )));
        }
        for (index, (x, width, height)) in children.into_iter().enumerate() {
            let entry = entries
                .iter()
                .find(|entry| entry.node() == index + 1)
                .ok_or_else(|| {
                    Error::new(format!("missing prepared Range child packet {index}"))
                })?;
            let y = if index % 2 == 0 {
                BLOCK_SHIFT
            } else {
                FIRST_BASELINE
            } as Scalar;
            let expected = layout::NodeOutput {
                location: layout::Point::new(x, y),
                size: layout::Size::new(width, height),
                content_size: layout::Size::new(width, height),
                ..layout::NodeOutput::with_source_index(layout::SourceIndex::new(index))
            };
            if entry.output() != expected {
                return Err(Error::new(format!(
                    "prepared Range child packet {index} changed: expected {expected:?}, got {:?}",
                    entry.output(),
                )));
            }
        }
    }
    Ok(())
}

fn metric_matches(actual: f64, expected: f64) -> bool {
    // Independent arithmetic and the placement recurrence may associate
    // operations differently. The raw box and Range fields remain exact.
    (actual - expected).abs() <= f64::EPSILON * 16.0 * expected.abs().max(1.0)
}

fn verify_selected_lines(text: &surgeist_text::Layout, rtl: bool) -> Result<(), Error> {
    let lines = text.lines();
    let expected = [(Range::new(0, 6), 0.0), (Range::new(6, 7), LINE_HEIGHT)];
    if lines.len() != expected.len() {
        return Err(Error::new("prepared Range selected line census changed"));
    }
    for (line, (range, top)) in lines.iter().zip(expected) {
        let rect = line.bounds();
        if line.range() != range
            || line.baseline() != FIRST_BASELINE + top
            || !metric_matches(rect.origin.y, top + BLOCK_SHIFT)
            || (rect.origin.x, rect.size.width, rect.size.height) != (0.0, 72.0, LINE_HEIGHT)
        {
            return Err(Error::new(format!(
                "prepared Range selected line changed: {line:?}"
            )));
        }
    }
    let glyphs: Vec<_> = text
        .glyph_runs()
        .into_iter()
        .flat_map(|run| {
            let origin = run.origin();
            run.glyphs()
                .iter()
                .map(|glyph| {
                    (
                        run.line(),
                        glyph.range(),
                        glyph.id(),
                        origin.x + glyph.pen() + f64::from(glyph.x()),
                        origin.y + f64::from(glyph.y()),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let x = if rtl {
        [24.0, 40.0, 56.0, 56.0]
    } else {
        [0.0, 16.0, 32.0, 0.0]
    };
    let expected: Vec<_> = [0, 2, 3, 6]
        .into_iter()
        .zip(x)
        .enumerate()
        .map(|(index, (start, x))| {
            let line = usize::from(index == 3);
            (
                line,
                Range::new(start, start + 1),
                58,
                x,
                FIRST_BASELINE + line as f64 * LINE_HEIGHT,
                FONT_SIZE as f32,
            )
        })
        .collect();
    if glyphs != expected {
        return Err(Error::new(format!(
            "prepared Range glyph placement changed: {glyphs:?}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTRACT: &str = include_str!("../../../../../docs/text-layout-contract.md");

    #[test]
    fn prepared_range_policy_verification_refuses_changed_source_anchor() {
        assert!(PolicyIdentity::from_contract(CONTRACT).is_ok());
        let changed = CONTRACT.replacen("post-L1/L2 position.", "pre-L1 position.", 1);
        assert_ne!(changed, CONTRACT);
        assert!(PolicyIdentity::from_contract(&changed).is_err());
        assert!(PolicyIdentity::from_contract("missing policy").is_err());
    }

    #[test]
    fn prepared_range_policy_rejects_a_shadowed_owner_section() {
        let extended = format!("{CONTRACT}\n### Unrelated owner section\n\nMore context.\n");
        assert!(PolicyIdentity::from_contract(&extended).is_ok());
        let shadowed =
            format!("{extended}\n### Nonempty Text range fragments\n\nConflicting policy.\n");
        assert!(PolicyIdentity::from_contract(&shadowed).is_err());
    }

    #[test]
    fn prepared_range_policy_rejects_shadowing_at_document_boundaries() {
        let rejected = [
            format!("### Nonempty Text range fragments\n\nConflicting policy.\n\n{CONTRACT}"),
            format!("{CONTRACT}\n### Nonempty Text range fragments"),
            format!("{CONTRACT}\n### Nonempty Text range fragments   \n\nConflicting policy.\n"),
        ]
        .map(|shadowed| PolicyIdentity::from_contract(&shadowed).is_err());
        assert_eq!(rejected, [true, true, true]);
    }

    #[test]
    fn prepared_range_policy_rejects_equivalent_atx_headings() {
        let rejected = [
            " ### Nonempty Text range fragments",
            "### Nonempty Text range fragments ###",
            "## Nonempty Text range fragments",
            "###\tNonempty Text range fragments",
            "### Nonempty text range fragments",
        ]
        .map(|heading| {
            let shadowed = format!("{heading}\n\nConflicting policy.\n\n{CONTRACT}");
            PolicyIdentity::from_contract(&shadowed).is_err()
        });
        assert_eq!(rejected, [true; 5]);
    }

    #[test]
    fn prepared_range_policy_reserves_the_title_and_anchor() {
        let rejected = [
            "Nonempty Text range fragments\n=============================",
            "### Nonempty **Text** range fragments",
            "### [Nonempty Text range fragments](#conflicting-policy)",
            "<a id=\"nonempty-text-range-fragments\"></a>",
        ]
        .map(|heading| {
            let shadowed = format!("{heading}\n\nConflicting policy.\n\n{CONTRACT}");
            PolicyIdentity::from_contract(&shadowed).is_err()
        });
        assert_eq!(rejected, [true; 4]);
    }

    #[test]
    fn prepared_range_verdict_identifies_the_adopted_policy_revision() {
        let original = include_str!(
            "captured/prepared-range-differences/xml/block/prepared_text_continuity_bidi__border_box_rtl.xml"
        );
        let golden = Golden::parse(original).unwrap();
        let mut raw = Vec::new();
        let policy = PolicyIdentity::from_contract(CONTRACT).unwrap();
        qualify(&golden, SOURCE, "border_box_rtl", &policy, &mut raw).unwrap();
        let verdict: serde_json::Value =
            serde_json::from_slice(raw.split(|&byte| byte == b'\n').nth(1).unwrap()).unwrap();
        assert_eq!(
            verdict["policy_sha256"],
            "ef6219d28d15d9131a45d0c48edef12436333a1e1f2868f54d9ec8847d09ada0"
        );
    }

    #[test]
    fn rejects_an_unexpected_rtl_match_and_a_later_range_difference() {
        let original = include_str!(
            "captured/prepared-range-differences/xml/block/prepared_text_continuity_bidi__border_box_rtl.xml"
        );
        let mut golden = Golden::parse(original).unwrap();
        let mut raw = Vec::new();
        let policy = PolicyIdentity::from_contract(CONTRACT).unwrap();
        qualify(&golden, SOURCE, "border_box_rtl", &policy, &mut raw).unwrap();

        for (owner, row, replacement) in [(1, 2, 24.0), (2, 0, 56.015625)] {
            let mut changed = golden.clone();
            changed.expectations.children[owner]
                .range_rectangles
                .as_mut()
                .unwrap()[row]
                .start = replacement;
            raw.clear();
            let error = qualify(&changed, SOURCE, "border_box_rtl", &policy, &mut raw).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("prepared Range comparison changed")
            );
            let output: serde_json::Value = serde_json::from_slice(&raw).unwrap();
            assert_eq!(output["ranges"].as_array().unwrap().len(), 5);
        }

        golden.prepared_range_frame = false;
        raw.clear();
        assert!(qualify(&golden, SOURCE, "border_box_rtl", &policy, &mut raw).is_err());
        assert!(raw.is_empty());
    }

    #[test]
    fn prepared_range_control_rows_keep_source_identity_when_coordinates_match_other_pieces() {
        let original = include_str!(
            "captured/prepared-range-differences/xml/block/prepared_text_continuity_bidi__border_box_rtl.xml"
        );
        let mut golden = Golden::parse(original).unwrap();
        let rows = golden.expectations.children[1]
            .range_rectangles
            .as_mut()
            .unwrap();
        // The retained row ordinals name RLI and PDI. Each altered browser
        // start is numerically equal to the other control's Native start.
        rows[0].start = 24.0;
        rows[2].start = 40.0;

        let policy = PolicyIdentity::from_contract(CONTRACT).unwrap();
        let mut raw = Vec::new();
        let error = qualify(&golden, SOURCE, "border_box_rtl", &policy, &mut raw).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("prepared Range comparison changed")
        );
        let output: serde_json::Value = serde_json::from_slice(&raw).unwrap();
        let ranges = output["ranges"].as_array().unwrap();
        assert_eq!(ranges.len(), 5);
        assert_eq!(
            ranges[2]["native_pieces"],
            serde_json::json!([
                {"unit": 1, "piece": 0, "local": [1, 2], "aggregate": [2, 3]},
                {"unit": 2, "piece": 0, "local": [2, 3], "aggregate": [3, 4]},
            ]),
            "the combined X rectangle retains both query pieces"
        );
        for (index, row, source, browser, native) in
            [(1, 0, [1, 2], 24.0, 40.0), (3, 2, [4, 5], 40.0, 24.0)]
        {
            assert_eq!(
                ranges[index],
                serde_json::json!({
                    "path": "/1",
                    "row": row,
                    "occurrence": 1,
                    "line": 0,
                    "edge": "left",
                    "native_source": source,
                    "native_pieces": [{"unit": if row == 0 { 0 } else { 3 }, "piece": 0,
                        "local": [source[0] - 1, source[1] - 1], "aggregate": source}],
                    "browser_start": browser,
                    "native_start": native,
                    "browser_advance": 0.0,
                    "native_advance": 0.0,
                })
            );
        }
    }
}
