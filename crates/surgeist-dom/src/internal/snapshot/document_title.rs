//! Caller-admitted title reads over the exact published ordinary tree.

use super::{Descendant, DocumentId, DomError, DomNode, DomNodeKind, DomSnapshot};
use surgeist_html_semantics::{HtmlDocumentTitleKind, HtmlDocumentTitleText};

impl DomSnapshot {
    /// Observe the normalized title of an exact Document. The first ordinary
    /// HTML title wins except under an SVG `svg` root, which uses its first
    /// direct SVG title. Only the selected title's direct Text contributes.
    ///
    /// `step` admits tree observations, name comparisons and input code units.
    /// `reserve` funds overlapping traversal scratch. `emit` owns output
    /// admission and storage. A successful absence emits nothing. Any first
    /// lookup, admission or output error is returned to the caller, even after
    /// `emit` has already received an accepted prefix.
    pub fn visit_document_title_with<E: From<DomError>, Backing>(
        &self,
        document: DocumentId,
        mut step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
        mut emit: impl FnMut(u16) -> Result<(), E>,
    ) -> Result<(), E> {
        let document = self.node_with(document.node(), &mut step)?;
        if !matches!(document.kind(), DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind.into());
        }
        let Some(root) = document.first_element_child_with(&mut step)? else {
            return Ok(());
        };
        let name = root.as_element().ok_or(DomError::WrongNodeKind)?.name();
        let kind = HtmlDocumentTitleKind::for_root(name, &mut step)?;
        let selected = match kind {
            HtmlDocumentTitleKind::Html if matches_title(root, kind, &mut step)? => Some(root),
            HtmlDocumentTitleKind::Html => {
                root.find_descendant_with(&mut step, reserve, |node, s| {
                    Ok(if matches_title(node, kind, s)? {
                        Descendant::Found(node)
                    } else {
                        Descendant::Descend
                    })
                })?
            }
            HtmlDocumentTitleKind::Svg => {
                let mut title = None;
                for &id in root.children() {
                    let child = self.node_with(id, &mut step)?;
                    if matches_title(child, kind, &mut step)? {
                        title = Some(child);
                        break;
                    }
                }
                title
            }
        };
        if let Some(title) = selected {
            let mut text = HtmlDocumentTitleText::default();
            title.visit_child_text_admitted(step, |data, step| {
                for &unit in data.code_units() {
                    step()?;
                    for unit in text.push(unit) {
                        emit(unit)?;
                    }
                }
                Ok(())
            })?;
        }
        Ok(())
    }
}

fn matches_title<E>(
    node: DomNode<'_>,
    kind: HtmlDocumentTitleKind,
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    let Some(element) = node.as_element() else {
        return Ok(false);
    };
    kind.matches(element.name(), step)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Dom, DomBatch,
        limits::{DomLimits, DomNameLimits},
    };
    use std::{cell::Cell, sync::atomic::AtomicBool};
    use surgeist_html::values::Namespace;

    #[test]
    fn document_title_observation_returns_first_refusal_and_releases_scratch() {
        let limits = DomLimits::new(
            32,
            2048,
            64,
            128,
            64,
            16,
            100_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let root = batch.create_element_ns(&document, Namespace::html(), "html");
        let container = batch.create_element_ns(&document, Namespace::html(), "body");
        let first = batch.create_element_ns(&document, Namespace::html(), "title");
        let later = batch.create_element_ns(&document, Namespace::html(), "title");
        let text = batch.create_text(&document, "  A  B  ");
        batch.insert_before(&root, &document, None);
        batch.insert_before(&container, &root, None);
        batch.insert_before(&later, &root, None);
        batch.insert_before(&first, &container, None);
        batch.insert_before(&text, &first, None);
        let document = dom
            .apply(batch, &AtomicBool::new(false))
            .unwrap()
            .document(&document)
            .unwrap();
        let snapshot = dom.snapshot();
        let refused = snapshot.visit_document_title_with(
            document,
            || Ok(()),
            |_| Err::<(), _>(DomError::AllocationFailed),
            |_| panic!("output before scratch admission"),
        );
        assert!(matches!(refused, Err(DomError::AllocationFailed)));

        struct Scratch<'a>(&'a Cell<bool>);
        impl Drop for Scratch<'_> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }
        let held = Cell::new(false);
        let mut accepted = Vec::new();
        let refused = snapshot.visit_document_title_with(
            document,
            || Ok(()),
            |_| {
                held.set(true);
                Ok(Scratch(&held))
            },
            |unit| {
                assert!(!held.get());
                if unit == u16::from(b' ') {
                    return Err(DomError::Cancelled);
                }
                accepted.push(unit);
                Ok(())
            },
        );
        assert!(matches!(refused, Err(DomError::Cancelled)));
        assert_eq!(accepted, [u16::from(b'A')]);
        assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
    }
}
