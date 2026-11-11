//! Replay of independent browser text measurements for the two Grid Lanes sources.

use super::{Error, FontFamily, Golden, Scalar, TestTree, flex_content_text::Proof, layout};
use crate::fri08_grid_lanes_text_profile as profile;

fn invalid(reason: impl std::fmt::Display) -> Error {
    Error::new(format!("Grid Lanes monospace text: {reason}"))
}

pub(super) fn install(tree: &mut TestTree, golden: &Golden, proof: &Proof) -> Result<(), Error> {
    let count = profile::WIDTHS.len() * profile::LABELS.len();
    if !proof.is_grid_lanes_text()
        || !proof.matches_source(golden)
        || proof.occurrences().len() != count
        || tree.nodes.len() != count + profile::WIDTHS.len() + 1
        || !tree.paragraphs.is_empty()
    {
        return Err(invalid("incomplete measured Grid Lanes source"));
    }

    for occurrence in proof.occurrences() {
        let (_, _, node) = occurrence
            .path
            .locate(tree)
            .ok_or_else(|| invalid("missing authored Flex leaf"))?;
        let leaf = &tree.nodes[node];
        let style = tree.box_node_input(node);
        let sizes = occurrence
            .sizes
            .ok_or_else(|| invalid("missing independent text sizes"))?;
        if !sizes.valid()
            || style.display != layout::Display::Flex
            || style.direction != occurrence.direction
            || style.writing_mode != layout::WritingMode::HorizontalTb
            || style.padding != layout::Edges::all(layout::Length::ZERO)
            || style.border != layout::Edges::all(layout::Length::ZERO)
            || leaf.font_family != FontFamily::Monospace
            || leaf.font_size != 15.0
            || leaf.line_height != 15.0
            || leaf.synthetic
            || !leaf.children.is_empty()
            || !leaf
                .text
                .as_deref()
                .is_some_and(|label| profile::LABELS.contains(&label))
            || tree.grid_lanes_text.insert(node, sizes).is_some()
        {
            return Err(invalid(format!(
                "measured source {} changed",
                occurrence.path
            )));
        }
    }
    Ok(())
}

pub(super) fn measure(
    sizes: profile::Sizes,
    input: layout::LeafMeasureInput,
) -> Result<layout::Size, Error> {
    if input.flow_axes().writing_mode() != layout::WritingMode::HorizontalTb {
        return Err(invalid("unsupported measured source writing mode"));
    }
    let known = input.known_content_size();
    let request = input.content_sizing_request().axes().width;
    let min = sizes.min_content.inline_size;
    let max = sizes.max_content.inline_size;
    let width = known.width.unwrap_or_else(|| match request.operation() {
        layout::SizingOperation::MinContent => min,
        layout::SizingOperation::MaxContent => max,
        layout::SizingOperation::Normal => request
            .space()
            .definite_size()
            .map_or(max, |available| available.get().clamp(min, max)),
    });
    let height = match known.height {
        Some(height) => height,
        None => block_size(sizes, width)?,
    };
    Ok(layout::Size::new(width, height))
}

fn block_size(sizes: profile::Sizes, width: Scalar) -> Result<Scalar, Error> {
    if width >= sizes.first_fit.inline_size {
        Ok(sizes.first_fit.block_size)
    } else if width >= sizes.zero.inline_size && width <= sizes.last_wrap.inline_size {
        // The exact source has one break between two words and no emergency
        // wrapping. The zero-width and last-wrap probes bound the two-line interval.
        Ok(sizes.last_wrap.block_size)
    } else {
        Err(invalid(format!("unmeasured width {width}")))
    }
}
