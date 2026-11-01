//! Complete-body L1 + Native empty-gap rows + the existing resolved-level L2.
//! Contextual UBA and original-source projection have their existing sole owners.
use super::*;
use crate::engine::{
    analysis,
    bidi::{self, LineReset},
};
use crate::protocol::inline_box::incidence::{SourceIncidence, visit_bidi_owner};
use crate::{BidiLevel, SuppliedInlineParticipant, SuppliedLine};
use icu_properties::{
    CodePointSetData,
    props::{BidiClass, BidiControl},
};

pub(super) fn derive(data: &mut FamilyData, ledger: &mut Ledger) -> Result<()> {
    let prepared = data.prepared.inner.clone();
    let a = &prepared.analyzed;
    let p = &a.paragraph;
    let topology = p.scope_topology.as_ref().expect("admitted complete family");
    let mut positions = Vec::new();
    for anchor in 0..p.anchor_order.len() {
        account::work(ledger, 1)?;
        account::push_fallible(
            ledger,
            &mut positions,
            analysis::anchor_atom(anchor, p, a.processed.source().boxes(), &a.projection),
        )?;
    }
    let mut owners = Vec::new();
    for (index, scope) in topology.capture.scopes.iter().enumerate() {
        account::work(ledger, 1)?;
        if scope.domain == topology.domain && relevant(data, scope.enter, scope.exit, ledger)? {
            account::push_fallible(ledger, &mut owners, Owner::Scope(index))?;
        }
    }
    for index in 0..data.native_boxes.len() {
        let (a, b) = data.interval(Owner::Native(index));
        if relevant(data, a, b, ledger)? {
            account::push_fallible(ledger, &mut owners, Owner::Native(index))?;
        }
    }
    // First real source incidence is only an ordering witness, never membership.
    let mut first = Vec::new();
    analysis::bidi_input::Input {
        typography: a.typography,
        projection: &a.projection,
        infos: &a.analysis.infos,
        boxes: a.processed.source().boxes(),
        paragraph: p,
        combination: a.combination.as_deref(),
        events: Some(&a.logical.events),
        anchor_positions: &positions,
    }
    .visit(ledger, |row, ledger| {
        account::work(ledger, 1)?;
        let section = analysis::structural::owner_section(
            row.owner,
            p,
            &a.sections,
            a.combination.as_deref(),
            ledger.as_mut(),
        )?;
        let start = data.row_owners.len();
        let mut at = None;
        visit_bidi_owner(&prepared, row.owner, ledger, &mut |site, ledger| {
            at = Some(at.map_or(site.position(), |v: usize| v.min(site.position())));
            let leaf = owner_at(data, site, ledger)?;
            account::work(ledger, data.row_owners.len() - start)?;
            if !data.row_owners[start..].contains(&leaf) {
                account::push_fallible(ledger, &mut data.row_owners, leaf)?;
            }
            Ok(())
        })?;
        let end = data.row_owners.len();
        let content = match row.owner {
            BidiOwner::Atom(i) => {
                !CodePointSetData::new::<BidiControl>().contains32(a.projection.atoms[i].code_point)
            }
            BidiOwner::Control(i) => p.controls[i]
                .kind
                .code_point()
                .is_some_and(|cp| !CodePointSetData::new::<BidiControl>().contains32(cp)),
            _ => true,
        };
        let level = match row.owner {
            BidiOwner::Atom(i) => a.analysis.levels[i],
            BidiOwner::Atomic(i) => a.analysis.box_levels[i],
            BidiOwner::Control(i) => a.analysis.control_levels[i],
            BidiOwner::ComputedLineBoundary(i) => a.analysis.computed_levels[i],
            BidiOwner::CombinedCell(i) => a.analysis.cell_levels[i],
        };
        account::push_fallible(ledger, &mut first, at)?;
        account::push_fallible(
            ledger,
            &mut data.rows,
            Row {
                source: RowSource::Bidi(row.owner),
                body: 0,
                level,
                content,
                section,
                reset: LineReset::from_class(row.class),
                ends_body: row.class == BidiClass::ParagraphSeparator,
                owners: start..end,
            },
        )
    })?;
    account::drop_vec(ledger, positions)?;
    // A terminal real box needs one opaque ordering row even with no glyphs.
    // Ancestors borrow that descendant incidence; they do not get duplicate rows.
    for &owner in &owners {
        account::work(ledger, 1)?;
        if !data.source(owner).is_inline() {
            continue;
        }
        let (begin, end) = data.interval(owner);
        let mut content = false;
        for row in &data.rows {
            account::work(ledger, 1)?;
            if row.content && member(data, row, owner, ledger)? {
                content = true;
                break;
            }
        }
        if content {
            continue;
        }
        let mut descendant = false;
        for &other in &owners {
            account::work(ledger, 1)?;
            let (c, d) = data.interval(other);
            if data.source(other).is_inline() && begin < c && d < end {
                descendant = true;
                break;
            }
        }
        if descendant {
            continue;
        }
        let section = analysis::structural::section_at_order(begin, &a.sections, ledger.as_mut())?
            .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
        let mut index = data.rows.len();
        for (i, row) in data.rows.iter().enumerate() {
            account::work(ledger, 1)?;
            if row.section > section
                || (row.section == section && first[i].is_some_and(|at| begin <= at))
            {
                index = i;
                break;
            }
        }
        let start = data.row_owners.len();
        account::push_fallible(ledger, &mut data.row_owners, owner)?;
        insert(
            ledger,
            &mut data.rows,
            index,
            Row {
                source: RowSource::Empty(owner),
                body: 0,
                level: a.analysis.base_level,
                content: true,
                section,
                reset: LineReset::Other,
                ends_body: false,
                owners: start..start + 1,
            },
        )?;
        insert(ledger, &mut first, index, Some(begin))?;
    }
    account::drop_vec(ledger, first)?;
    let mut start = 0;
    while start < data.rows.len() {
        let section = data.rows[start].section;
        let mut end = start;
        while end < data.rows.len() && data.rows[end].section == section {
            account::work(ledger, 1)?;
            let delimiter = data.rows[end].ends_body;
            end += 1;
            if delimiter {
                break;
            }
        }
        body(data, start..end, ledger)?;
        start = end;
    }
    // Real demanded empty complete bodies remain an observable admitted family.
    if data.rows.is_empty() {
        body(data, 0..0, ledger)?;
    }
    let root = Owner::Scope(topology.capture.domain_scopes[topology.domain]);
    for body in 0..data.bodies.len() {
        let visual = data.bodies[body].visual.clone();
        account::push_fallible(
            ledger,
            &mut data.fragments,
            Fragment {
                owner: root,
                body,
                visual: visual.clone(),
                parent: None,
            },
        )?;
        for &owner in &owners {
            if owner == root {
                continue;
            }
            let mut open = None;
            for rank in visual.clone() {
                account::work(ledger, 1)?;
                let row = &data.rows[data.order[rank]];
                if !row.content {
                    continue;
                }
                if member(data, row, owner, ledger)? {
                    if open.is_none() {
                        open = Some(rank);
                    }
                } else if let Some(first) = open.take() {
                    account::push_fallible(
                        ledger,
                        &mut data.fragments,
                        Fragment {
                            owner,
                            body,
                            visual: first..rank,
                            parent: None,
                        },
                    )?;
                }
            }
            if let Some(first) = open {
                account::push_fallible(
                    ledger,
                    &mut data.fragments,
                    Fragment {
                        owner,
                        body,
                        visual: first..visual.end,
                        parent: None,
                    },
                )?;
            }
        }
    }
    for index in 0..data.fragments.len() {
        account::work(ledger, 1)?;
        let child = &data.fragments[index];
        let parent = data.parent(child.owner, ledger)?;
        let mut found = None;
        if let Some(parent) = parent {
            for (candidate, row) in data.fragments.iter().enumerate() {
                account::work(ledger, 1)?;
                if row.owner == parent
                    && row.body == child.body
                    && row.visual.start <= child.visual.start
                    && child.visual.end <= row.visual.end
                {
                    found = Some(candidate);
                    break;
                }
            }
            if found.is_none() {
                return Err(invalid(InlineBoxFragmentationError::InvalidIncidence));
            }
        }
        data.fragments[index].parent = found;
    }
    for (row, value) in data.rows.iter().enumerate() {
        account::work(ledger, 1)?;
        match value.source {
            RowSource::Bidi(owner) => {
                account::push_fallible(ledger, &mut data.row_lookup, (owner, row))?
            }
            RowSource::Empty(owner) => {
                account::push_fallible(ledger, &mut data.empty_lookup, (owner, row))?
            }
        }
    }
    crate::processing::contextual::alignment::operations::order_by(
        &mut data.row_lookup,
        ledger.as_mut(),
        |a, b| a.0.cmp(&b.0),
    )?;
    crate::processing::contextual::alignment::operations::order_by(
        &mut data.empty_lookup,
        ledger.as_mut(),
        |a, b| a.0.cmp(&b.0),
    )?;
    for (fragment, f) in data.fragments.iter().enumerate() {
        for rank in f.visual.clone() {
            account::work(ledger, 1)?;
            let row = data.order[rank];
            if member(data, &data.rows[row], f.owner, ledger)? {
                account::push_fallible(ledger, &mut data.row_fragments, (row, fragment))?;
            }
        }
    }
    crate::processing::contextual::alignment::operations::order_by(
        &mut data.row_fragments,
        ledger.as_mut(),
        |a, b| a.cmp(b),
    )?;
    account::drop_vec(ledger, owners)
}
pub(super) fn scope_at(data: &FamilyData, site: SourceIncidence) -> Result<Owner> {
    let topology = data
        .prepared
        .inner
        .analyzed
        .paragraph
        .scope_topology
        .as_ref()
        .expect("complete captured family");
    let scope = topology
        .capture
        .events
        .get(site.position())
        .and_then(|event| event.before)
        .unwrap_or(topology.capture.domain_scopes[topology.domain]);
    if topology.capture.scopes[scope].domain != topology.domain {
        return Err(invalid(InlineBoxFragmentationError::InvalidIncidence));
    }
    Ok(Owner::Scope(scope))
}
fn member(data: &FamilyData, row: &Row, owner: Owner, ledger: &mut Ledger) -> Result<bool> {
    for &leaf in &data.row_owners[row.owners.clone()] {
        let mut at = Some(leaf);
        while let Some(value) = at {
            account::work(ledger, 1)?;
            if value == owner {
                return Ok(true);
            }
            at = data.parent(value, ledger)?;
        }
    }
    Ok(false)
}
fn body(data: &mut FamilyData, logical: Range<usize>, ledger: &mut Ledger) -> Result<()> {
    let base = data.prepared.inner.analyzed.analysis.base_level;
    let mut levels = Vec::new();
    let mut roles = Vec::new();
    for row in &data.rows[logical.clone()] {
        account::work(ledger, 1)?;
        if matches!(row.source, RowSource::Bidi(_)) {
            account::push_fallible(ledger, &mut levels, row.level)?;
            account::push_fallible(ledger, &mut roles, row.reset)?;
        }
    }
    account::work(ledger, levels.len().saturating_mul(3))?;
    bidi::reset_line_levels(&mut levels, &roles, base);
    let body = data.bodies.len();
    let mut next = 0;
    for row in &mut data.rows[logical.clone()] {
        account::work(ledger, 1)?;
        row.body = body;
        if matches!(row.source, RowSource::Bidi(_)) {
            row.level = levels[next];
            next += 1;
        }
    }
    account::drop_vec(ledger, levels)?;
    account::drop_vec(ledger, roles)?;
    let mut following = base;
    for row in data.rows[logical.clone()].iter_mut().rev() {
        account::work(ledger, 1)?;
        match row.source {
            RowSource::Empty(_) => row.level = following,
            RowSource::Bidi(_) => following = row.level,
        }
    }
    let mut slots = Vec::new();
    let mut minimum_odd: Option<u8> = None;
    let mut maximum = 0;
    for row in &data.rows[logical.clone()] {
        account::work(ledger, 1)?;
        let level = row.level;
        maximum = maximum.max(level);
        if level % 2 == 1 {
            minimum_odd = Some(minimum_odd.map_or(level, |value| value.min(level)));
        }
        account::push_fallible(
            ledger,
            &mut slots,
            SuppliedInlineParticipant::Boundary {
                level: BidiLevel::try_new(level)
                    .map_err(|_| invalid(InlineBoxFragmentationError::InvalidIncidence))?,
            },
        )?;
    }
    let mut order = account::filled_fallible(ledger, slots.len(), 0)?;
    let passes = minimum_odd.map_or(0, |minimum| usize::from(maximum - minimum) + 1);
    // The existing L2 owner initializes order, scans minimum/optional maximum,
    // then scans and reverses per level. Admit its full bound before entry;
    // discovering our bound above shares the already-required slot visit.
    let work = slots.len().checked_mul(3 + 4 * passes).ok_or_else(|| {
        crate::engine::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
            kind: crate::ScopeAlignmentLimitKind::StructuralWork,
        })
    })?;
    account::work(ledger, work)?;
    SuppliedLine::new(&slots)
        .reorder_into(&mut order)
        .map_err(|_| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
    let begin = data.order.len();
    for index in order.iter().copied() {
        account::push_fallible(ledger, &mut data.order, logical.start + index)?;
    }
    let end = data.order.len();
    account::push_fallible(ledger, &mut data.bodies, Body { visual: begin..end })?;
    account::drop_vec(ledger, order)?;
    account::drop_vec(ledger, slots)
}

pub(super) fn owner_at(
    data: &FamilyData,
    site: SourceIncidence,
    ledger: &mut Ledger,
) -> Result<Owner> {
    let leaf = scope_at(data, site)?;
    if let Some(index) = data.native_at(site, ledger)? {
        let (a, b) = data.interval(Owner::Native(index));
        let (p, q) = data.interval(leaf);
        if p < a && b < q {
            return Ok(Owner::Native(index));
        }
    }
    Ok(leaf)
}

fn relevant(data: &FamilyData, begin: usize, end: usize, ledger: &mut Ledger) -> Result<bool> {
    for section in &data.prepared.inner.analyzed.sections {
        account::work(ledger, 1)?;
        let span = section.span.expect("admitted captured complete section");
        if begin <= span.exit && span.entry <= end {
            return Ok(true);
        }
    }
    Ok(false)
}

/// One initialized new row plus the actual existing rows shifted by insertion.
/// Work and off-side capacity growth are admitted before rotating any old row.
fn insert<T>(ledger: &mut Ledger, values: &mut Vec<T>, index: usize, value: T) -> Result<()> {
    let shifted = values
        .len()
        .checked_sub(index)
        .ok_or_else(|| invalid(InlineBoxFragmentationError::InvalidIncidence))?;
    account::work(ledger, shifted)?;
    account::push_fallible(ledger, values, value)?;
    values[index..].rotate_right(1);
    Ok(())
}
