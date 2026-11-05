//! Portable admission for the captured Ahem Grid leaf profile.

use super::{Error, Node, NodeKind, element_scroll_sizes, pinned_font};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MeasuredLeaf {
    pub(crate) child_index: usize,
}

fn invalid(detail: &str) -> Error {
    Error::new(format!("invalid measured-leaf: {detail}"))
}

pub(super) fn parse(
    test: roxmltree::Node<'_, '_>,
    root: &Node,
    element_sizes: Option<&element_scroll_sizes::Table>,
) -> Result<Option<MeasuredLeaf>, Error> {
    let mut sections = test
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "measured-leaf");
    let Some(section) = sections.next() else {
        return Ok(None);
    };
    if sections.next().is_some()
        || section.parent() != Some(test)
        || section.tag_name().namespace().is_some()
        || element_sizes.is_none()
        || root.kind != NodeKind::Div
        || root.style.get("display") != Some("grid")
    {
        return Err(invalid(
            "requires one root Grid witness and its complete element scroll-size table",
        ));
    }
    let fixed = [
        ("profile", "ahem-10px-zwsp"),
        (
            "font-resource-sha256",
            pinned_font::AHEM_STYLESHEET_WOFF2_SHA256,
        ),
        ("first-advance", "40"),
        ("second-advance", "40"),
        ("full-advance", "80"),
    ];
    if section.attributes().len() != fixed.len() + 1
        || section.attributes().any(|attr| attr.namespace().is_some())
        || fixed
            .iter()
            .any(|(name, value)| section.attribute(*name) != Some(*value))
        || section.children().any(|node| {
            node.is_element() || node.text().is_some_and(|text| !text.trim().is_empty())
        })
    {
        return Err(invalid(
            "unknown payload or changed profile, font resource or advances",
        ));
    }
    let path = section
        .attribute("node-path")
        .ok_or_else(|| invalid("missing node-path"))?;
    let child_index = path
        .strip_prefix("0.")
        .and_then(|index| index.parse::<usize>().ok())
        .filter(|index| path == format!("0.{index}"))
        .ok_or_else(|| invalid("expected canonical direct Grid child path"))?;
    let leaf = root
        .children
        .get(child_index)
        .ok_or_else(|| invalid("source child missing"))?;
    if leaf.kind != NodeKind::Text
        || !leaf.children.is_empty()
        || leaf.text.as_deref() != Some("HHHH\u{200b}HHHH")
        || root.text.is_some()
        || root
            .children
            .iter()
            .enumerate()
            .any(|(index, child)| index != child_index && has_text(child))
    {
        return Err(invalid(
            "source does not identify the single captured text leaf",
        ));
    }
    Ok(Some(MeasuredLeaf { child_index }))
}

fn has_text(node: &Node) -> bool {
    node.text.is_some() || node.children.iter().any(has_text)
}
