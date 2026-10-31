//! Canonical graph construction with optional operation-local accounting.
use super::*;
use crate::processing::contextual::alignment::operations;

pub(super) fn needed(input: Input<'_>, ledger: &mut Ledger) -> Result<bool> {
    let mut ranges = Vec::new();
    let mut found = false;
    input.visit_cells(ledger, |binding, _, atoms, ledger| {
        found |= input.tracking(binding, ledger)? != 0.0;
        account::push(ledger, &mut ranges, atoms)
    })?;
    operations::order_by(&mut ranges, ledger.as_mut(), |a, b| {
        (a.start, a.end).cmp(&(b.start, b.end))
    })?;
    if found {
        account::drop_vec(ledger, ranges)?;
        return Ok(true);
    }
    let mut cell = 0;
    for (atom, (_, style)) in input.analysis.infos.iter().enumerate() {
        account::work(ledger, 1)?;
        while cell < ranges.len() {
            account::work(ledger, 1)?;
            if ranges[cell].end <= atom {
                cell += 1;
            } else {
                break;
            }
        }
        if let Some(range) = ranges.get(cell) {
            account::work(ledger, 1)?;
            if range.contains(&atom) {
                continue;
            }
        }
        if input.styles[*style].style.letter_spacing != 0.0 {
            account::drop_vec(ledger, ranges)?;
            return Ok(true);
        }
    }
    account::drop_vec(ledger, ranges)?;
    for b in input.boxes {
        account::work(ledger, 1)?;
        if b.letter_spacing() != 0.0 {
            return Ok(true);
        }
    }
    Ok(false)
}

fn excluded(
    exclusions: Option<&crate::engine::layout::data::OrdinalMap<Exclusion>>,
    atom: usize,
    ledger: &mut Ledger,
) -> Result<bool> {
    let Some(exclusions) = exclusions else {
        return Ok(false);
    };
    Ok(
        operations::binary_search_by(&exclusions.rows, ledger.as_mut(), |row| row.0.cmp(&atom))?
            .is_ok(),
    )
}

fn operation_graphemes(
    input: Input<'_>,
    exclusions: Option<&crate::engine::layout::data::OrdinalMap<Exclusion>>,
    ledger: &mut Ledger,
) -> Result<Vec<Range<usize>>> {
    let mut cells = Vec::new();
    input.visit_cells(ledger, |_, _, atoms, ledger| {
        account::push(ledger, &mut cells, atoms)?;
        Ok(())
    })?;
    operations::order_by(&mut cells, ledger.as_mut(), |a, b| {
        (a.start, a.end).cmp(&(b.start, b.end))
    })?;
    let mut components = account::clone_vec(ledger, &input.projection.barrier_atoms)?;
    account::push(ledger, &mut components, 0)?;
    account::push(ledger, &mut components, input.projection.atoms.len())?;
    for cell in &cells {
        account::work(ledger, 1)?;
        account::push(ledger, &mut components, cell.start)?;
        account::push(ledger, &mut components, cell.end)?;
    }
    operations::order_by(&mut components, ledger.as_mut(), Ord::cmp)?;
    account::work(ledger, components.len())?;
    components.dedup();
    let representatives = account::collect(ledger, input.projection.segmentation.chars())?;
    let mut output: Vec<Range<usize>> = Vec::new();
    let mut cell = 0;
    for component in components.windows(2) {
        account::work(ledger, 1)?;
        while cell < cells.len() {
            account::work(ledger, 1)?;
            if cells[cell].end <= component[0] {
                cell += 1;
            } else {
                break;
            }
        }
        if let Some(range) = cells.get(cell) {
            account::work(ledger, 1)?;
            if range.contains(&component[0]) {
                continue;
            }
        }
        let mut text = Vec::new();
        let mut kept = Vec::new();
        for (offset, representative) in representatives[component[0]..component[1]]
            .iter()
            .enumerate()
        {
            account::work(ledger, 1)?;
            let atom = component[0] + offset;
            if !excluded(exclusions, atom, ledger)? {
                account::push(ledger, &mut kept, (text.len(), atom))?;
                let mut bytes = [0; 4];
                for byte in representative.encode_utf8(&mut bytes).as_bytes() {
                    account::push(ledger, &mut text, *byte)?;
                }
            }
        }
        let text_ref =
            std::str::from_utf8(&text).expect("canonical projection representatives are UTF-8");
        let mut stops = Vec::new();
        for stop in icu_segmenter::GraphemeClusterSegmenter::new().segment_str(text_ref) {
            account::push(ledger, &mut stops, stop)?;
        }
        for pair in stops.windows(2) {
            account::work(ledger, 1)?;
            let first =
                operations::partition_point(&kept, ledger.as_mut(), |(byte, _)| *byte < pair[0])?;
            let end =
                operations::partition_point(&kept, ledger.as_mut(), |(byte, _)| *byte < pair[1])?;
            if first == end {
                continue;
            }
            let a = kept[first].1;
            let b = kept[end - 1].1;
            let start =
                operations::partition_point(&input.analysis.graphemes, ledger.as_mut(), |g| {
                    g.end <= a
                })?;
            let last =
                operations::partition_point(&input.analysis.graphemes, ledger.as_mut(), |g| {
                    g.end <= b
                })?;
            let range = input.analysis.graphemes[start].start.max(component[0])
                ..input.analysis.graphemes[last].end.min(component[1]);
            let mut has_exclusion = false;
            if let Some(exclusions) = exclusions {
                let at = operations::partition_point(&exclusions.rows, ledger.as_mut(), |row| {
                    row.0 < range.start
                })?;
                account::work(ledger, 1)?;
                has_exclusion = exclusions.rows.get(at).is_some_and(|row| row.0 < range.end);
            }
            if has_exclusion {
                if let Some(prior) = output.last_mut()
                    && prior.end > range.start
                {
                    prior.end = prior.end.max(range.end);
                } else {
                    account::push(ledger, &mut output, range)?;
                }
            } else {
                for g in &input.analysis.graphemes[start..=last] {
                    account::work(ledger, 1)?;
                    let g = g.start.max(component[0])..g.end.min(component[1]);
                    if output.last() != Some(&g) {
                        account::push(ledger, &mut output, g)?;
                    }
                }
            }
        }
        account::drop_vec(ledger, stops)?;
        account::drop_vec(ledger, text)?;
        account::drop_vec(ledger, kept)?;
    }
    account::drop_vec(ledger, cells)?;
    account::drop_vec(ledger, components)?;
    account::drop_vec(ledger, representatives)?;
    Ok(output)
}

pub(super) fn order_levels(levels: &[u8], ledger: &mut Ledger) -> Result<Vec<usize>> {
    let mut slots = Vec::new();
    let mut min_odd: Option<u8> = None;
    let mut maximum = 0;
    for &level in levels {
        account::work(ledger, 1)?;
        maximum = maximum.max(level);
        if level & 1 != 0 {
            min_odd = Some(min_odd.map_or(level, |x| x.min(level)));
        }
        let level = crate::BidiLevel::try_new(level).map_err(|_| {
            crate::Error::new(crate::ErrorCode::LayoutFailed, "character unit level")
        })?;
        account::push(
            ledger,
            &mut slots,
            crate::SuppliedInlineParticipant::Boundary { level },
        )?;
    }
    let mut order = account::filled(ledger, levels.len(), 0)?;
    let passes = min_odd.map_or(0, |n| usize::from(maximum - n) + 1);
    let work = levels.len().checked_mul(passes * 4 + 3).ok_or_else(|| {
        crate::engine::scope_alignment::refusal(crate::ScopeAlignmentError::Overflow {
            kind: crate::ScopeAlignmentLimitKind::StructuralWork,
        })
    })?;
    account::work(ledger, work)?;
    crate::SuppliedLine::new(&slots)
        .reorder_into(&mut order)
        .map_err(|_| crate::Error::new(crate::ErrorCode::LayoutFailed, "character unit order"))?;
    account::drop_vec(ledger, slots)?;
    Ok(order)
}

fn reorder_units(units: &mut [Unit], ledger: &mut Ledger) -> Result<()> {
    let levels = account::collect(ledger, units.iter().map(|u| u.level))?;
    let order = order_levels(&levels, ledger)?;
    let mut target = account::filled(ledger, units.len(), 0)?;
    for (visual, &logical) in order.iter().enumerate() {
        account::work(ledger, 1)?;
        target[logical] = visual;
    }
    for i in 0..units.len() {
        while target[i] != i {
            account::work(ledger, 4)?;
            let j = target[i];
            units.swap(i, j);
            target.swap(i, j);
        }
    }
    account::drop_vec(ledger, target)?;
    account::drop_vec(ledger, order)?;
    account::drop_vec(ledger, levels)
}

fn source_unit(
    input: Input<'_>,
    atoms: Range<usize>,
    exclusions: Option<&crate::engine::layout::data::OrdinalMap<Exclusion>>,
    domains: &[Option<usize>],
    ledger: &mut Ledger,
) -> Result<Option<Unit>> {
    let mut eligible = Vec::new();
    let mut retained = Vec::new();
    for i in atoms.clone() {
        account::work(ledger, 1)?;
        if !excluded(exclusions, i, ledger)? {
            account::push(ledger, &mut eligible, i)?;
            account::push(ledger, &mut retained, input.projection.atoms[i].clone())?;
        }
    }
    if eligible.is_empty() {
        account::drop_vec(ledger, eligible)?;
        account::drop_vec(ledger, retained)?;
        return Ok(None);
    }
    account::work(ledger, retained.len())?;
    let base = unit_properties::base(&retained).expect("nonempty presentation EGC");
    let mut local = 0;
    for (i, atom) in retained.iter().enumerate() {
        account::work(ledger, 1)?;
        if std::ptr::eq(atom, base) {
            local = i;
            break;
        }
    }
    let base_index = eligible[local];
    let style = &input.styles[input.analysis.infos[base_index].1].style;
    let mut scripts = Vec::new();
    let properties = unit_properties::visit_properties(&retained, |visit| match visit {
        unit_properties::PropertyVisit::Inspect => account::work(ledger, 1),
        unit_properties::PropertyVisit::Script(s) => account::push(ledger, &mut scripts, s),
    })?;
    let mut real = Vec::new();
    for &script in &scripts {
        account::work(ledger, 1)?;
        if !matches!(script, Script::Common | Script::Inherited | Script::Unknown) {
            account::push(ledger, &mut real, script)?;
        }
    }
    let family = style.locale.as_ref().and_then(locale_family);
    let unspecified = style.locale.as_ref().is_none_or(|l| l.as_str() == "und");
    let numeric = GeneralCategoryGroup::Number.contains(properties.category);
    let letter = GeneralCategoryGroup::Letter.contains(properties.category) || numeric;
    let symbol = GeneralCategoryGroup::Symbol.contains(properties.category)
        || GeneralCategoryGroup::Punctuation.contains(properties.category);
    let mut all_supported = !real.is_empty();
    let mut all_cursive = true;
    let mut all_plain = true;
    for &s in &real {
        account::work(ledger, 1)?;
        all_supported &= supported(s);
        all_cursive &= cursive(s);
        all_plain &= !cursive(s);
    }
    let uniform_rule = !letter || all_cursive || all_plain;
    let mut resolved_scripts = account::clone_vec(ledger, &real)?;
    let mut qualified = if unspecified {
        all_supported && uniform_rule
            || (numeric || properties.category == GeneralCategory::SpaceSeparator)
                && real.is_empty()
    } else {
        family.is_some() && (all_supported && uniform_rule || real.is_empty())
    };
    let ambiguous = (letter || symbol) && real.len() > 1 && (!all_supported || !all_plain);
    let mut justify_qualified = qualified;
    if ambiguous {
        account::drop_vec(ledger, resolved_scripts)?;
        resolved_scripts = Vec::new();
        if let Some(family) = family {
            for &s in &real {
                account::work(ledger, family.len() + 1)?;
                if family.contains(&s) {
                    account::push(ledger, &mut resolved_scripts, s)?;
                }
            }
        }
        let mut coherent = !resolved_scripts.is_empty();
        let mut plain = true;
        for &s in &resolved_scripts {
            account::work(ledger, 1)?;
            coherent &= supported(s);
            plain &= !cursive(s);
        }
        coherent &= plain || resolved_scripts.len() == 1;
        justify_qualified = coherent;
        if letter {
            qualified = coherent;
        } else {
            qualified |= coherent;
        }
    }
    let mut is_cursive = letter && !resolved_scripts.is_empty();
    let mut resolved_cursive = !resolved_scripts.is_empty();
    for &s in &resolved_scripts {
        account::work(ledger, 1)?;
        resolved_cursive &= cursive(s);
    }
    is_cursive &= resolved_cursive;
    let justify_cursive = is_cursive || symbol && resolved_cursive;
    let domain = domains.get(base_index).copied().flatten();
    let mut native_only = true;
    let mut barrier = false;
    for a in &retained {
        account::work(ledger, 1)?;
        native_only &=
            CodePointSetData::new::<DefaultIgnorableCodePoint>().contains32(a.code_point);
        barrier |= matches!(a.code_point, 9 | 10 | 13 | 0x2028 | 0x2029);
    }
    let context_scripts = account::clone_vec(ledger, family.unwrap_or_default())?;
    let atom_ranges = account::collect(ledger, [atoms])?;
    account::drop_vec(ledger, real)?;
    account::drop_vec(ledger, retained)?;
    account::drop_vec(ledger, eligible)?;
    Ok(Some(Unit {
        subject: Subject::Text,
        order: (0, 0),
        atoms: atom_ranges,
        atomics: Vec::new(),
        base: Some(base_index),
        left: style.letter_spacing,
        right: style.letter_spacing,
        qualified,
        cursive: is_cursive,
        justify_cursive,
        justify_qualified,
        symbol,
        resolved_scripts,
        scripts,
        context_scripts,
        content_context: false,
        domain,
        barrier,
        native_only,
        level: input.analysis.levels[base_index],
    }))
}

pub(crate) fn build(
    input: Input<'_>,
    potential: bool,
    exclusions: Option<&crate::engine::layout::data::OrdinalMap<Exclusion>>,
    ledger: &mut Ledger,
) -> Result<Graph> {
    let mut native = None;
    let metered = input.has_cells();
    let ledger = if metered {
        storage::require_ledger(ledger)?;
        ledger
    } else {
        &mut native
    };
    if !potential && !needed(input, ledger)? {
        return Ok(Graph::default());
    }
    let mut units = Vec::new();
    let mut domains = Vec::new();
    if let Some(p) = input.presentation.filter(|p| !p.domains.is_empty()) {
        domains = account::filled(ledger, input.projection.atoms.len(), None)?;
        for (d, domain) in p.domains.iter().enumerate() {
            account::work(ledger, 1)?;
            for unit in &p.units[domain.units.clone()] {
                account::work(ledger, 1)?;
                for atom in unit.atoms.clone() {
                    account::work(ledger, 1)?;
                    if domains[atom].is_none() {
                        domains[atom] = Some(d);
                    }
                }
            }
        }
    }
    let graphemes = operation_graphemes(input, exclusions, ledger)?;
    for atoms in &graphemes {
        account::work(ledger, 1)?;
        if let Some(unit) = source_unit(input, atoms.clone(), exclusions, &domains, ledger)? {
            account::push(ledger, &mut units, unit)?;
        }
    }
    account::drop_vec(ledger, graphemes)?;
    account::drop_vec(ledger, domains)?;
    let mut present = Vec::new();
    if let Some(p) = input.presentation {
        present = account::filled(ledger, input.boxes.len(), false)?;
        for u in &p.units {
            account::work(ledger, 1)?;
            if let crate::protocol::prepared::PresentationUnitKind::Atomic { occurrence } = u.kind {
                present[occurrence] = true;
            }
        }
    }
    for (occurrence, b) in input.boxes.iter().enumerate() {
        account::work(ledger, 1)?;
        if b.kind() == InlineBoxKind::OutOfFlow {
            continue;
        }
        if input.presentation.is_some() && !present[occurrence] {
            continue;
        }
        let atomics = account::collect(ledger, [occurrence])?;
        account::push(
            ledger,
            &mut units,
            Unit {
                subject: Subject::Atomics,
                order: (0, 0),
                atoms: Vec::new(),
                atomics,
                base: None,
                left: b.letter_spacing(),
                right: b.letter_spacing(),
                qualified: true,
                cursive: false,
                justify_cursive: false,
                justify_qualified: true,
                symbol: false,
                resolved_scripts: Vec::new(),
                scripts: Vec::new(),
                context_scripts: Vec::new(),
                content_context: false,
                domain: None,
                barrier: false,
                native_only: false,
                level: input.analysis.box_levels[occurrence],
            },
        )?;
    }
    account::drop_vec(ledger, present)?;
    input.visit_cells(ledger, |binding, level, _, ledger| {
        let tracking = input.tracking(binding, ledger)?;
        account::push(
            ledger,
            &mut units,
            Unit {
                subject: Subject::Cell(binding),
                order: (0, 0),
                atoms: Vec::new(),
                atomics: Vec::new(),
                base: None,
                left: tracking,
                right: tracking,
                qualified: true,
                cursive: false,
                justify_cursive: false,
                justify_qualified: true,
                symbol: false,
                resolved_scripts: Vec::new(),
                scripts: Vec::new(),
                context_scripts: Vec::new(),
                content_context: false,
                domain: None,
                barrier: false,
                native_only: false,
                level,
            },
        )
    })?;
    resolve_context(&mut units, input, ledger)?;
    let mut selected_members = Vec::new();
    if let Some(p) = input.presentation {
        let order = crate::protocol::prepared::presentation::unit_order(
            &p.units,
            &p.domains,
            input.boxes,
            ledger,
        )?;
        let mut rank = account::filled(ledger, p.units.len(), 0)?;
        for (i, &u) in order.iter().enumerate() {
            account::work(ledger, 1)?;
            rank[u] = i;
            let unit = &p.units[u];
            let member = match unit.kind {
                crate::protocol::prepared::PresentationUnitKind::Cell { cell } => {
                    Some(Member::Cell(input.cell_binding(cell)))
                }
                crate::protocol::prepared::PresentationUnitKind::Atomic { occurrence } => {
                    (input.boxes[occurrence].kind() == InlineBoxKind::InFlow)
                        .then_some(Member::Atomic(occurrence))
                }
                _ => (!unit.atoms.is_empty()).then(|| Member::Text(unit.atoms.clone())),
            };
            if let Some(member) = member {
                account::push(ledger, &mut selected_members, member)?;
            }
        }
        let mut atom_units = account::filled(ledger, input.projection.atoms.len(), None)?;
        let mut atomic_units = account::filled(ledger, input.boxes.len(), None)?;
        let mut cell_units = account::filled(ledger, input.binding_count(), None)?;
        for (index, punit) in p.units.iter().enumerate() {
            account::work(ledger, 1)?;
            match punit.kind {
                crate::protocol::prepared::PresentationUnitKind::Cell { cell } => {
                    cell_units[input.cell_binding(cell)] = Some(index)
                }
                crate::protocol::prepared::PresentationUnitKind::Atomic { occurrence } => {
                    atomic_units[occurrence] = Some(index)
                }
                _ => {
                    for atom in punit.atoms.clone() {
                        account::work(ledger, 1)?;
                        atom_units[atom] = Some(index);
                    }
                }
            }
        }
        for unit in &mut units {
            account::work(ledger, 1)?;
            let index = match unit.subject {
                Subject::Text => atom_units[unit.base.expect("text base")],
                Subject::Atomics => atomic_units[unit.atomics[0]],
                Subject::Cell(binding) => cell_units[binding],
            }
            .expect("selected unit retains actual source custody");
            let sub = unit.base.unwrap_or(0);
            unit.order = (
                rank[index],
                if p.units[index].line_level & 1 == 0 {
                    sub
                } else {
                    usize::MAX - sub
                },
            );
        }
        account::drop_vec(ledger, atom_units)?;
        account::drop_vec(ledger, atomic_units)?;
        account::drop_vec(ledger, cell_units)?;
        account::drop_vec(ledger, rank)?;
        account::drop_vec(ledger, order)?;
    } else {
        input.set_original_order(&mut units, ledger)?;
    }
    // The final tiebreaker is the actual construction ordinal, preserving the
    // native stable-key order without an allocating std stable sort.
    let mut keyed = Vec::new();
    for (ordinal, unit) in units.drain(..).enumerate() {
        account::push(ledger, &mut keyed, (unit.order, ordinal, unit))?;
    }
    account::drop_vec(ledger, units)?;
    operations::order_by(&mut keyed, ledger.as_mut(), |a, b| {
        (a.0, a.1).cmp(&(b.0, b.1))
    })?;
    let mut units = Vec::new();
    for (_, _, unit) in keyed.drain(..) {
        account::push(ledger, &mut units, unit)?;
    }
    account::drop_vec(ledger, keyed)?;
    if input.presentation.is_none() {
        let mut start = 0;
        for end in 0..=units.len() {
            account::work(ledger, 1)?;
            if end == units.len() || units[end].barrier {
                reorder_units(&mut units[start..end], ledger)?;
                start = end.saturating_add(1);
            }
        }
    }
    let mut grouped: Vec<Unit> = Vec::new();
    for mut unit in units.drain(..) {
        account::work(ledger, 1)?;
        if unit.subject == Subject::Atomics
            && let Some(prior) = grouped.last_mut()
            && prior.subject == Subject::Atomics
        {
            prior.right = unit.right;
            for atomic in unit.atomics.drain(..) {
                account::push(ledger, &mut prior.atomics, atomic)?;
            }
            storage::drop_unit(unit, ledger)?;
        } else {
            account::push(ledger, &mut grouped, unit)?;
        }
    }
    account::drop_vec(ledger, units)?;
    let mut boundaries = Vec::new();
    for i in 1..grouped.len() {
        account::work(ledger, 1)?;
        if grouped[i - 1].barrier || grouped[i].barrier {
            continue;
        }
        let same = grouped[i - 1].resolved_scripts.len() == 1
            && grouped[i].resolved_scripts.len() == 1
            && grouped[i - 1].resolved_scripts == grouped[i].resolved_scripts;
        let protected = same && grouped[i - 1].cursive && grouped[i].cursive;
        let justify = same && grouped[i - 1].justify_cursive && grouped[i].justify_cursive;
        let a = grouped[i - 1].right;
        let b = grouped[i].left;
        let authored = if protected || a == -b {
            0.0
        } else {
            a.midpoint(b)
        };
        account::push(
            ledger,
            &mut boundaries,
            Boundary {
                before: i - 1,
                after: i,
                authored,
                mean_lost: !protected && a != -b && authored == 0.0,
                geometry: if justify {
                    SpacingBoundaryGeometry::ProtectedCursive
                } else {
                    SpacingBoundaryGeometry::Independent
                },
            },
        )?;
    }
    let index = super::index::Index::new(&grouped, ledger)?;
    let mut graph = Graph {
        units: grouped,
        boundaries,
        potential,
        regions: Vec::new(),
        ownership: if metered {
            GraphOwnership::CellScratch
        } else {
            GraphOwnership::Native
        },
        index,
    };
    if input.presentation.is_some() {
        graph.regions = graph.find_regions_accounted(&selected_members, ledger)?;
    }
    account::drop_vec(ledger, selected_members)?;
    Ok(graph)
}

fn resolve_context(units: &mut [Unit], input: Input<'_>, ledger: &mut Ledger) -> Result<()> {
    let count = input.presentation.map_or(1, |p| p.domains.len() + 1);
    let mut domains = Vec::new();
    for _ in 0..count {
        account::push(ledger, &mut domains, Vec::<(Script, usize)>::new())?;
    }
    for unit in units.iter() {
        account::work(ledger, 1)?;
        let mut real = None;
        let mut many = false;
        for &s in &unit.scripts {
            account::work(ledger, 1)?;
            if !matches!(s, Script::Common | Script::Inherited | Script::Unknown) {
                if real.is_some() {
                    many = true;
                } else {
                    real = Some(s);
                }
            }
        }
        if !many && let Some(script) = real {
            let counts = &mut domains[unit.domain.map_or(0, |d| d + 1)];
            match operations::binary_search_by(counts, ledger.as_mut(), |row| row.0.cmp(&script))? {
                Ok(i) => counts[i].1 += 1,
                Err(i) => {
                    account::reserve_vec(ledger, counts, 1)?;
                    account::work(ledger, counts.len() - i + 1)?;
                    if let Some(b) = ledger {
                        b.add_records(1)?;
                    }
                    counts.insert(i, (script, 1));
                }
            }
        }
    }
    for i in 0..units.len() {
        account::work(ledger, units[i].scripts.len() + 1)?;
        if !units[i].symbol
            || units[i].base.is_none()
            || units[i]
                .scripts
                .iter()
                .any(|s| !matches!(*s, Script::Common | Script::Inherited | Script::Unknown))
        {
            continue;
        }
        units[i].content_context = true;
        let counts = &domains[units[i].domain.map_or(0, |d| d + 1)];
        let mut maximum = 0;
        for row in counts {
            account::work(ledger, 1)?;
            maximum = maximum.max(row.1);
        }
        let mut actual = Vec::new();
        for &(s, count) in counts {
            account::work(ledger, 1)?;
            if count == maximum {
                account::push(ledger, &mut actual, s)?;
            }
        }
        if actual.is_empty() {
            for &s in &units[i].context_scripts {
                account::push(ledger, &mut actual, s)?;
            }
        }
        let mut any_cursive = false;
        for &s in &actual {
            account::work(ledger, 1)?;
            any_cursive |= cursive(s);
        }
        if actual.len() > 1 && any_cursive {
            let mut selected = Vec::new();
            for &s in &actual {
                account::work(ledger, units[i].context_scripts.len() + 1)?;
                if units[i].context_scripts.contains(&s) {
                    account::push(ledger, &mut selected, s)?;
                }
            }
            if selected.len() == 1 {
                account::drop_vec(ledger, actual)?;
                actual = selected;
            } else {
                account::drop_vec(ledger, selected)?;
            }
        }
        let mut supported_all = !actual.is_empty();
        let mut cursive_all = true;
        let mut plain_all = true;
        for &s in &actual {
            account::work(ledger, 1)?;
            supported_all &= supported(s);
            cursive_all &= cursive(s);
            plain_all &= !cursive(s);
        }
        let uniform = supported_all && (cursive_all || plain_all);
        let actual_cursive = uniform && cursive_all;
        let mut declared_cursive = true;
        let mut declared_plain = true;
        for &s in &units[i].context_scripts {
            account::work(ledger, 1)?;
            declared_cursive &= cursive(s);
            declared_plain &= !cursive(s);
        }
        let declared_uniform =
            units[i].context_scripts.is_empty() || declared_cursive || declared_plain;
        let conflict = !units[i].context_scripts.is_empty()
            && (!declared_uniform || declared_cursive != actual_cursive);
        let base = units[i].base.expect("text symbol base");
        let unspecified = input.styles[input.analysis.infos[base].1]
            .style
            .locale
            .as_ref()
            .is_none_or(|l| l.as_str() == "und");
        units[i].qualified = uniform && (unspecified || !units[i].context_scripts.is_empty());
        units[i].justify_qualified =
            units[i].qualified && !conflict && (!actual_cursive || actual.len() == 1);
        units[i].justify_cursive = actual_cursive;
        account::drop_vec(
            ledger,
            std::mem::replace(&mut units[i].resolved_scripts, actual),
        )?;
    }
    for rows in &mut domains {
        account::drop_vec(ledger, std::mem::take(rows))?;
    }
    account::drop_vec(ledger, domains)?;
    Ok(())
}
