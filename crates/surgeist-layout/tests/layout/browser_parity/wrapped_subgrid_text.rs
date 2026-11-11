//! Text measurements for the source-qualified wrapped Subgrid leaves.

use std::collections::{BTreeMap, btree_map::Entry};

use surgeist_text as text;

use super::{
    Error, FontFamily, InheritedTextContext, LineHeightState, Scalar, TestTree,
    compute_actual_batch, flex_content_text, layout, pinned_font,
};
use crate::fri08_wrapped_subgrid_profile as profile;

pub(super) struct Measurer {
    system: text::System,
    face: surgeist_font::FontFace,
    paragraphs: BTreeMap<(usize, bool), TestTree>,
    layouts: BTreeMap<(usize, bool, Width), Placement>,
}

impl std::fmt::Debug for Measurer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WrappedSubgridMeasurer")
            .field("face", &self.face)
            .field("cached_layouts", &self.layouts.len())
            .finish_non_exhaustive()
    }
}

fn invalid(detail: impl std::fmt::Display) -> Error {
    Error::new(format!("Wrapped Subgrid Text: {detail}"))
}

impl Measurer {
    pub(super) fn new() -> Result<Self, Error> {
        let resource = surgeist_font::FontResource::decode(
            pinned_font::AHEM_STYLESHEET_WOFF2.to_vec(),
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .map_err(invalid)?;
        let face = resource
            .face(0)
            .filter(|_| resource.face_count() == 1)
            .ok_or_else(|| invalid("missing exact stylesheet face"))?;
        let mut system = text::System::default();
        system.register_font_resource(resource).map_err(invalid)?;
        Ok(Self {
            system,
            face,
            paragraphs: BTreeMap::new(),
            layouts: BTreeMap::new(),
        })
    }

    fn place(
        &mut self,
        label: &str,
        direction: layout::Direction,
        width: Width,
    ) -> Result<&Placement, Error> {
        let label_index = profile::LABELS
            .iter()
            .position(|&known| label == known)
            .ok_or_else(|| invalid("unexpected authored text"))?;
        let rtl = direction == layout::Direction::Rtl;
        let flow = layout::FlowAxes::new(layout::WritingMode::HorizontalTb, direction);
        let key = (label_index, rtl, width);
        if let Entry::Vacant(entry) = self.layouts.entry(key) {
            let paragraph = match self.paragraphs.entry((label_index, rtl)) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let prepared = flex_content_text::prepare_label(
                        &mut self.system,
                        &self.face,
                        label,
                        direction,
                    )?;
                    entry.insert(paragraph_tree(label, flow, prepared)?)
                }
            };
            let (space, operation) = match width {
                Width::MinContent => (
                    layout::AvailableSpace::Indefinite,
                    layout::SizingOperation::MinContent,
                ),
                Width::MaxContent => (
                    layout::AvailableSpace::Indefinite,
                    layout::SizingOperation::MaxContent,
                ),
                Width::Definite(bits) => (
                    layout::AvailableSpace::Definite(
                        layout::NonNegativeFiniteOf::new(Scalar::from_bits(bits))
                            .map_err(invalid)?,
                    ),
                    layout::SizingOperation::Normal,
                ),
            };
            let axes = layout::Size::new(
                layout::AxisSizingRequest::new(space, operation),
                layout::AxisSizingRequest::new(
                    layout::AvailableSpace::Indefinite,
                    layout::SizingOperation::Normal,
                ),
            );
            let fallback = layout::OrthogonalFallback::new(
                layout::Size::NONE,
                layout::NearestScrollport::Absent,
                layout::Size::splat(layout::OrthogonalFallbackLimit::Unbounded),
            );
            let request = layout::LayoutRootRequest::standalone(
                layout::SizingRequest::new(axes, fallback),
                flow,
                axes.map(|axis| axis.space().definite_size()),
            );
            let batch = compute_actual_batch(paragraph, "wrapped Subgrid paragraph", request)?;
            paragraph.apply_completed_batch(&batch);
            let [placed] = batch.placed_text() else {
                return Err(invalid("expected one actual placed paragraph"));
            };
            let placed = Placement {
                size: paragraph.nodes[0].unrounded.size,
                layout: placed.layout().clone(),
            };
            entry.insert(placed);
        }
        Ok(&self.layouts[&key])
    }

    pub(super) fn measure(
        &mut self,
        label: &str,
        input: layout::LeafMeasureInput,
    ) -> Result<layout::Size, Error> {
        let flow = input.flow_axes();
        if flow.writing_mode() != layout::WritingMode::HorizontalTb {
            return Err(invalid("unsupported text writing mode"));
        }
        let direction = flow.direction();
        let known = input.known_content_size();
        let inline = input.content_sizing_request().axes().width;
        let width = match known.width {
            Some(width) => width,
            None => {
                let measure = |measurer: &mut Self, width| {
                    measurer
                        .place(label, direction, width)
                        .map(|placed| placed.size.width)
                };
                match inline.operation() {
                    layout::SizingOperation::MinContent => measure(self, Width::MinContent)?,
                    layout::SizingOperation::MaxContent => measure(self, Width::MaxContent)?,
                    layout::SizingOperation::Normal => {
                        let max = measure(self, Width::MaxContent)?;
                        if let Some(available) = inline.space().definite_size() {
                            max.min(available.get())
                                .max(measure(self, Width::MinContent)?)
                        } else {
                            max
                        }
                    }
                }
            }
        };
        let height = match known.height {
            Some(height) => height,
            None => {
                self.place(label, direction, Width::Definite(width.to_bits()))?
                    .size
                    .height
            }
        };
        Ok(layout::Size::new(width, height))
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Width {
    MinContent,
    MaxContent,
    Definite(u32),
}

struct Placement {
    size: layout::Size,
    layout: text::Layout,
}

// Measure a single anonymous block from the same prepared paragraph the Flex
// leaf would use. Cache each sizing request outside the nested Grid traversal.
fn paragraph_tree(
    label: &str,
    flow: layout::FlowAxes,
    prepared: text::PreparedText,
) -> Result<TestTree, Error> {
    let origin = prepared
        .origin_views()
        .next()
        .ok_or_else(|| invalid("missing prepared source occurrence"))?
        .handle();
    let inherited = InheritedTextContext {
        font_family: FontFamily::Ahem,
        font_size: 10.0,
        line_height: LineHeightState::Px(10.0),
        grid_lanes_text: false,
        inline_level_text: false,
        containing_flow: Some(flow),
        parent: None,
        range_root: None,
    };
    let mut tree = TestTree::default();
    let root = tree.push_synthetic_text_wrapper(Vec::new(), inherited);
    let node = tree.push_synthetic_text(
        label,
        InheritedTextContext {
            parent: Some(root),
            ..inherited
        },
    )?;
    tree.nodes[node].layout_input =
        layout::LayoutInput::inline_text(layout::TextParticipation::new(origin.clone()));
    tree.nodes[node].text = None;
    tree.nodes[root].children.push(node);
    tree.paragraphs.push(layout::TextParagraphBinding {
        inline_box_fragments: layout::InlineBoxFragmentationDemand::NotRequested,
        prepared,
        container: root,
        nodes: vec![node],
        origins: vec![layout::TextOriginBinding { node, origin }],
        atomics: Vec::new(),
        controls: Vec::new(),
        line_strut: None,
    });
    Ok(tree)
}

pub(super) fn install(
    tree: &mut super::TestTree,
    golden: &super::Golden,
    proof: &super::flex_content_text::Proof,
) -> Result<(), Error> {
    if !proof.is_wrapped_subgrid()
        || !proof.matches_source(golden)
        || tree.nodes.len() != 13
        || proof.occurrences().len() != 6
        || !tree.paragraphs.is_empty()
    {
        return Err(invalid("incomplete wrapped Subgrid source custody"));
    }
    for occurrence in proof.occurrences() {
        let (_, _, node) = occurrence
            .path
            .locate(tree)
            .ok_or_else(|| invalid("wrapped source path is absent"))?;
        let leaf = &tree.nodes[node];
        let style = leaf
            .layout_input
            .as_box()
            .ok_or_else(|| invalid("wrapped text requires its authored Flex box"))?;
        if style.display != layout::Display::Flex
            || style.direction != occurrence.direction
            || style.writing_mode != layout::WritingMode::HorizontalTb
            || style.padding != layout::Edges::all(layout::Length::ZERO)
            || style.border != layout::Edges::all(layout::Length::ZERO)
            || leaf.font_family != super::FontFamily::Ahem
            || leaf.font_size != 10.0
            || leaf.line_height != 10.0
            || leaf.synthetic
            || !leaf.children.is_empty()
            || !leaf
                .text
                .as_deref()
                .is_some_and(|label| profile::LABELS.contains(&label))
        {
            return Err(invalid(format!(
                "source {} changed before measurement",
                occurrence.path
            )));
        }
    }
    tree.wrapped_subgrid_text = Some(std::rc::Rc::new(std::cell::RefCell::new(Measurer::new()?)));
    Ok(())
}

pub(super) fn verify(
    tree: &super::TestTree,
    proof: &super::flex_content_text::Proof,
) -> Result<(), Error> {
    let mut measurer = tree
        .wrapped_subgrid_text
        .as_ref()
        .ok_or_else(|| invalid("missing exact Text measurement session"))?
        .borrow_mut();
    for occurrence in proof.occurrences() {
        let (_, _, node) = occurrence
            .path
            .locate(tree)
            .ok_or_else(|| invalid("missing measured source path"))?;
        let leaf = &tree.nodes[node];
        let label = leaf
            .text
            .as_deref()
            .ok_or_else(|| invalid("missing source text"))?;
        let placement = &measurer
            .place(
                label,
                occurrence.direction,
                Width::Definite(leaf.unrounded.size.width.to_bits()),
            )?
            .layout;
        let line_count = occurrence.words.last().map_or(0, |range| range.line + 1);
        if placement.metrics().line_count() != line_count {
            return Err(invalid(format!(
                "{}: browser {line_count} lines, Native {}",
                occurrence.path,
                placement.metrics().line_count()
            )));
        }
        let mut origin = [0.0, 0.0];
        let mut ancestor = node;
        while let Some(parent) = tree.nodes[ancestor].parent {
            let location = tree.nodes[ancestor].final_layout.location;
            origin[0] += f64::from(location.x);
            origin[1] += f64::from(location.y);
            ancestor = parent;
        }
        for observed in &occurrence.words {
            let selected = placement
                .selection(text::Selection::new(
                    text::Cursor::new(observed.start, text::Affinity::After),
                    text::Cursor::new(observed.end, text::Affinity::Before),
                ))
                .map_err(invalid)?;
            let [selection] = selected.rects() else {
                return Err(invalid(format!(
                    "{}: source range {}..{} has unexpected Native fragments",
                    occurrence.path, observed.start, observed.end
                )));
            };
            let rect = selection.rect();
            let native = [
                rect.origin.x + origin[0],
                rect.origin.y + origin[1],
                rect.size.width,
                rect.size.height,
            ];
            let browser = [observed.x, observed.y, observed.width, observed.height];
            if selection.line() != observed.line || native != browser {
                return Err(invalid(format!(
                    "{}: source range {}..{} browser {:?} line {}, Native {:?} line {}",
                    occurrence.path,
                    observed.start,
                    observed.end,
                    browser,
                    observed.line,
                    native,
                    selection.line(),
                )));
            }
        }
        if let Some(clip) = occurrence.clip {
            let path = occurrence.path.to_string();
            super::element_scroll_sizes::compare_projected_box_sizes(&path, leaf, clip)?;
            let geometry = leaf
                .unrounded
                .scroll_geometry
                .ok_or_else(|| invalid("missing actual hidden-leaf scroll geometry"))?;
            let port = geometry.scrollport();
            let actual = geometry.overflow_clip();
            if geometry.used_overflow_x() != layout::Overflow::Hidden
                || geometry.used_overflow_y() != layout::Overflow::Hidden
                || actual.x().map(|edge| [edge.minimum(), edge.maximum()])
                    != Some([port.origin().x, port.origin().x + port.size().width])
                || actual.y().map(|edge| [edge.minimum(), edge.maximum()])
                    != Some([port.origin().y, port.origin().y + port.size().height])
            {
                return Err(invalid(format!(
                    "{path}: hidden clip disagrees with the actual client frame"
                )));
            }
        }
    }
    Ok(())
}

pub(super) fn compare(
    golden: &super::Golden,
) -> std::result::Result<
    (
        Vec<String>,
        std::result::Result<super::exact_differences::QualifiedComparison, String>,
    ),
    super::exact_differences::QualificationError,
> {
    use super::exact_differences::{
        QualificationError, QualifiedComparison, QualifiedVerdict, RawVerdict,
    };

    if golden
        .flex_content_text
        .as_ref()
        .is_none_or(|proof| !proof.is_wrapped_subgrid())
    {
        return Err(QualificationError::IdentityChanged {
            component: "missing wrapped Subgrid text proof".into(),
        });
    }
    let mut numeric = super::NumericComparisonSink::CollectNumeric(Vec::new());
    super::compare_captured_with_batch(golden, &mut numeric, |tree, batch| {
        if tree.nodes.len() != 13
            || tree
                .nodes
                .iter()
                .any(|node| !node.unrounded_present || !node.final_layout_present)
            || !batch.placed_text().is_empty()
        {
            return Err(invalid("incomplete authored Native box census"));
        }
        Ok(())
    })
    .map_err(|error| QualificationError::ComparisonFailure {
        reason: error.to_string(),
    })?;
    let super::NumericComparisonSink::CollectNumeric(differences) = numeric else {
        unreachable!("numeric comparison owns its collecting sink");
    };
    let fields: Vec<_> = differences
        .iter()
        .map(|field| format!("{field:?}"))
        .collect();
    let verdict = if fields.is_empty() {
        Ok(QualifiedComparison {
            raw: RawVerdict::Matches,
            qualified: QualifiedVerdict::MatchingControl,
        })
    } else {
        Err(QualificationError::DifferenceSetChanged {
            missing: Vec::new(),
            extra: fields.clone(),
            changed: Vec::new(),
        }
        .to_string())
    };
    Ok((fields, verdict))
}
