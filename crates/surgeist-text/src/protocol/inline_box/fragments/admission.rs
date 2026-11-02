//! Complete topology/body admission before deriving any global fragments.
use super::*;
use crate::processing::contextual::alignment::operations;
use crate::{ScopeRole, SourceProcessingTopology};

pub(super) fn prepare(
    prepared: &PreparedText,
    mut native_boxes: Vec<InlineBoxSource>,
    allowance: Option<ScopeAlignmentAllowance>,
) -> Result<PreparedInlineBoxFamily> {
    let topology = prepared
        .inner
        .analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .ok_or_else(|| super::super::invalid(super::super::InlineBoxEdgeError::MissingCapture))?;
    let mut ledger = crate::engine::scope_alignment::census::begin_inline_box_family(
        &prepared.inner,
        &native_boxes,
        allowance,
    )?;
    let domain = topology.owner.domain();
    account::work(&mut ledger, 1)?;
    if domain.domain_topology() != SourceProcessingTopology::Complete {
        return Err(invalid(InlineBoxFragmentationError::IncompleteTopology {
            domain: domain.index,
        }));
    }
    for (section, row) in prepared.inner.analyzed.sections.iter().enumerate() {
        account::work(&mut ledger, 1)?;
        let span = row.span.ok_or_else(|| {
            super::super::invalid(super::super::InlineBoxEdgeError::MissingCapture)
        })?;
        for (paragraph, original) in topology.capture.paragraphs.iter().enumerate() {
            account::work(&mut ledger, 1)?;
            if original.domain != topology.domain
                || original.exit <= span.entry
                || span.exit <= original.enter
            {
                continue;
            }
            let mut owned = false;
            for member in 0..topology.owner.len() {
                account::work(&mut ledger, 1)?;
                owned |= topology.owner.member(member).index == paragraph;
            }
            if !owned {
                return Err(invalid(
                    InlineBoxFragmentationError::IncompleteBodyCoverage { section, paragraph },
                ));
            }
        }
    }
    let mut index = 0;
    while index < native_boxes.len() {
        account::work(&mut ledger, 1)?;
        let canonical = match &native_boxes[index].kind {
            InlineBoxSourceKind::Captured(scope) => {
                if !Arc::ptr_eq(&scope.capture, &topology.capture) {
                    return Err(super::super::invalid(
                        super::super::InlineBoxEdgeError::ForeignCapture,
                    ));
                }
                if scope.domain().index != domain.index {
                    return Err(super::super::invalid(
                        super::super::InlineBoxEdgeError::WrongDomain,
                    ));
                }
                if scope.role() != ScopeRole::Inline {
                    return Err(super::super::invalid(
                        super::super::InlineBoxEdgeError::InvalidOwner,
                    ));
                }
                native_boxes[index].clone()
            }
            InlineBoxSourceKind::NativePair { start, end } => {
                prepared.native_inline_box(start.clone(), end.clone())?
            }
        };
        if matches!(canonical.kind, InlineBoxSourceKind::Captured(_)) {
            account::work(&mut ledger, native_boxes.len() - index)?;
            native_boxes.remove(index);
        } else {
            native_boxes[index] = canonical;
            index += 1;
        }
    }
    operations::order_by(&mut native_boxes, ledger.as_mut(), |a, b| {
        let (a, z) = a.interval();
        let (b, y) = b.interval();
        (a.index, z.index).cmp(&(b.index, y.index))
    })?;
    let mut index = 1;
    while index < native_boxes.len() {
        account::work(&mut ledger, 1)?;
        if native_boxes[index - 1] == native_boxes[index] {
            account::work(&mut ledger, native_boxes.len() - index)?;
            native_boxes.remove(index);
        } else {
            index += 1;
        }
    }
    for (index, source) in native_boxes.iter().enumerate() {
        let (a, b) = source.interval();
        for other in &native_boxes[..index] {
            account::work(&mut ledger, 1)?;
            let (c, d) = other.interval();
            if crossing(a.index, b.index, c.index, d.index) {
                return Err(super::super::invalid(
                    super::super::InlineBoxEdgeError::InvalidPair,
                ));
            }
        }
        for scope in &topology.capture.scopes {
            account::work(&mut ledger, 1)?;
            if scope.domain == topology.domain
                && scope.definition.is_none()
                && crossing(a.index, b.index, scope.enter, scope.exit)
            {
                return Err(super::super::invalid(
                    super::super::InlineBoxEdgeError::InvalidPair,
                ));
            }
        }
    }
    let mut data = FamilyData {
        prepared: prepared.clone(),
        native_boxes,
        native_ancestors: Vec::new(),
        captured_native_parents: Vec::new(),
        rows: Vec::new(),
        bodies: Vec::new(),
        order: Vec::new(),
        fragments: Vec::new(),
        row_owners: Vec::new(),
        row_lookup: Vec::new(),
        empty_lookup: Vec::new(),
        endpoint_fragments: Vec::new(),
        row_fragments: Vec::new(),
        retention: None,
        usage: None,
    };
    data.admit_native_overlay(&mut ledger)?;
    build::derive(&mut data, &mut ledger)?;
    account::header::<FamilyData>(&mut ledger)?;
    let mut family = PreparedInlineBoxFamily {
        inner: Arc::new(data),
    };
    if let Some(mut budget) = ledger {
        let retention = crate::engine::scope_alignment::census::retained_inline_box_family(
            &family,
            &mut budget,
        )?;
        let data = Arc::get_mut(&mut family.inner).expect("family has not escaped construction");
        data.retention = Some(retention);
        data.usage = Some(budget.usage());
    }
    Ok(family)
}

fn crossing(a: usize, b: usize, c: usize, d: usize) -> bool {
    (a < c && c < b && b < d) || (c < a && a < d && d < b)
}
