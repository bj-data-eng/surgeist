//! Browser parent-local locations and world-corner rounded sizes, never Native operands.
use super::{Error, Node};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub struct WorldRoundedLayout {
    pub node_path: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub scroll_size: Option<(f64, f64)>,
    pub client_size: Option<(f64, f64)>,
}
fn boxed(node: &Node) -> bool {
    node.inline_text.is_none()
        && node.prepared_occurrence.is_none()
        && node.inline_boundary.is_none()
}
fn paths(node: &Node, path: &str, out: &mut BTreeSet<String>) {
    if boxed(node) {
        out.insert(path.to_owned());
    }
    for (index, child) in node.children.iter().enumerate() {
        paths(child, &format!("{path}.{index}"), out);
    }
}
fn scalar(row: roxmltree::Node<'_, '_>, field: &str, extent: bool) -> Result<f64, Error> {
    let raw = row
        .attribute(field)
        .ok_or_else(|| Error::new(format!("missing diagnostic {field}")))?;
    let value: f64 = raw
        .parse()
        .map_err(|_| Error::new(format!("invalid diagnostic {field}")))?;
    if !value.is_finite() || extent && value < 0.0 {
        return Err(Error::new(format!("invalid diagnostic {field}")));
    }
    Ok(value)
}
fn pair(row: roxmltree::Node<'_, '_>, a: &str, b: &str) -> Result<Option<(f64, f64)>, Error> {
    match (row.attribute(a), row.attribute(b)) {
        (None, None) => Ok(None),
        (Some(_), Some(_)) => Ok(Some((scalar(row, a, true)?, scalar(row, b, true)?))),
        _ => Err(Error::new("diagnostic extent pair is incomplete")),
    }
}
pub(super) fn parse(
    test: roxmltree::Node<'_, '_>,
    root: &Node,
    version: Option<&str>,
) -> Result<Vec<WorldRoundedLayout>, Error> {
    let tables: Vec<_> = test
        .children()
        .filter(|node| node.has_tag_name("diagnostics"))
        .collect();
    if version != Some("4") {
        if !tables.is_empty() {
            return Err(Error::new(
                "world-rounding diagnostics require measurement version4",
            ));
        }
        return Ok(Vec::new());
    }
    if tables.len() != 1 {
        return Err(Error::new("V4 requires exactly one diagnostics table"));
    }
    let table = tables[0];
    if table.attributes().next().is_some()
        || table
            .children()
            .any(|node| node.is_text() && node.text().is_some_and(|value| !value.trim().is_empty()))
    {
        return Err(Error::new("invalid diagnostics table payload"));
    }
    let mut expected = BTreeSet::new();
    paths(root, "0", &mut expected);
    let mut seen = BTreeSet::new();
    let mut records = Vec::new();
    for row in table.children().filter(roxmltree::Node::is_element) {
        if !row.has_tag_name("world-rounded-layout")
            || row.children().any(|node| {
                node.is_element() || node.text().is_some_and(|value| !value.trim().is_empty())
            })
        {
            return Err(Error::new(
                "invalid world-rounded-layout diagnostic payload",
            ));
        }
        for attribute in row.attributes() {
            if ![
                "node-path",
                "x",
                "y",
                "width",
                "height",
                "scroll-width",
                "scroll-height",
                "client-width",
                "client-height",
            ]
            .contains(&attribute.name())
            {
                return Err(Error::new(format!(
                    "unexpected diagnostic field {}",
                    attribute.name()
                )));
            }
        }
        let path = row
            .attribute("node-path")
            .ok_or_else(|| Error::new("missing diagnostic node-path"))?;
        // The set is derived from parsed Layout children, excluding XML metadata.
        // Its canonical paths reject foreign, stale, duplicate and non-box rows.
        if !expected.contains(path) || !seen.insert(path.to_owned()) {
            return Err(Error::new("invalid or duplicate diagnostic node-path"));
        }
        records.push(WorldRoundedLayout {
            node_path: path.to_owned(),
            x: scalar(row, "x", false)?,
            y: scalar(row, "y", false)?,
            width: scalar(row, "width", true)?,
            height: scalar(row, "height", true)?,
            scroll_size: pair(row, "scroll-width", "scroll-height")?,
            client_size: pair(row, "client-width", "client-height")?,
        });
    }
    if seen != expected {
        return Err(Error::new("V4 diagnostics do not cover every actual box"));
    }
    Ok(records)
}
