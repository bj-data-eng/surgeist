//! Scratch measurement counters: ledger calls, list pushes, reservations.
use std::{cell::RefCell, collections::BTreeMap, panic::Location};

#[derive(Default, Clone)]
pub(crate) struct ListCounts {
    pub(crate) ops: u64,
    pub(crate) pushes: u64,
    pub(crate) exact_pushes: u64,
    pub(crate) push_bytes: u64,
    pub(crate) pops: u64,
    pub(crate) extends: u64,
    pub(crate) extend_bytes: u64,
    pub(crate) reservations: u64,
    pub(crate) reserved_new_bytes: u64,
    pub(crate) reserved_old_bytes: u64,
    pub(crate) ledger_work: u64,
    pub(crate) ledger_charge: u64,
    pub(crate) ledger_check: u64,
    pub(crate) ledger_changes: u64,
    pub(crate) ledger_live: u64,
}

#[derive(Default)]
struct State {
    label: Option<&'static str>,
    site: Option<&'static Location<'static>>,
    rsite: Option<(&'static Location<'static>, bool)>,
    rows: BTreeMap<String, u64>,
    rsites: BTreeMap<(&'static str, u32, bool), ListCounts>,
    lists: BTreeMap<&'static str, ListCounts>,
    sites: BTreeMap<(&'static str, u32), ListCounts>,
    work_calls: u64,
    charge_calls: u64,
    check_calls: u64,
    live_calls: u64,
    changes: u64,
    checkpoints: u64,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

pub(crate) struct Label(Option<&'static str>);
impl Label {
    pub(crate) fn enter(name: &'static str) -> Self {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            let prev = s.label.replace(name);
            s.lists.entry(name).or_default().ops += 1;
            Label(prev)
        })
    }
}
impl Drop for Label {
    fn drop(&mut self) {
        STATE.with(|s| s.borrow_mut().label = self.0);
    }
}

pub(crate) fn site(location: &'static Location<'static>) {
    STATE.with(|s| s.borrow_mut().site = Some(location));
}

pub(crate) fn site_if_unset(location: &'static Location<'static>) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.site.is_none() {
            s.site = Some(location);
        }
    });
}

pub(crate) fn rsite(location: &'static Location<'static>, geometric: bool) {
    STATE.with(|s| s.borrow_mut().rsite = Some((location, geometric)));
}

pub(crate) fn reservation(old: u64, new: u64) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let st = &mut *s;
        let c = if let Some(label) = st.label {
            st.lists.entry(label).or_default()
        } else if let Some((loc, g)) = st.rsite.take() {
            st.rsites.entry((loc.file(), loc.line(), g)).or_default()
        } else {
            st.rsites.entry(("<unattributed>", 0, false)).or_default()
        };
        c.reservations += 1;
        c.reserved_new_bytes += new;
        c.reserved_old_bytes += old;
    });
}

pub(crate) fn row(name: String) {
    STATE.with(|s| *s.borrow_mut().rows.entry(name).or_default() += 1);
}

pub(crate) fn checkpoint() {
    STATE.with(|s| s.borrow_mut().checkpoints += 1);
}

pub(crate) fn list(f: impl FnOnce(&mut ListCounts)) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let label = s.label.unwrap_or("<unlabeled>");
        f(s.lists.entry(label).or_default());
    });
}

/// Kind: 0 work, 1 charge, 2 check, 3 live.
pub(crate) fn ledger(kind: u8, changes: usize) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        let st = &mut *s;
        match kind {
            0 => st.work_calls += 1,
            1 => st.charge_calls += 1,
            2 => st.check_calls += 1,
            _ => st.live_calls += 1,
        }
        st.changes += changes as u64;
        let apply = |c: &mut ListCounts| {
            match kind {
                0 => c.ledger_work += 1,
                1 => c.ledger_charge += 1,
                2 => c.ledger_check += 1,
                _ => c.ledger_live += 1,
            }
            c.ledger_changes += changes as u64;
        };
        if let Some(label) = st.label {
            apply(st.lists.entry(label).or_default());
        } else if let Some(loc) = st.site.take() {
            apply(st.sites.entry((loc.file(), loc.line())).or_default());
        } else {
            apply(st.sites.entry(("<unattributed>", 0)).or_default());
        }
    });
}

pub(crate) fn reset() {
    STATE.with(|s| *s.borrow_mut() = State::default());
}

pub(crate) fn report() -> String {
    use std::fmt::Write as _;
    STATE.with(|s| {
        let s = s.borrow();
        let mut out = String::new();
        let _ = writeln!(
            out,
            "ledger calls: work={} charge={} check={} live_reads={} total_apply_check={} changes={} checkpoints={}",
            s.work_calls,
            s.charge_calls,
            s.check_calls,
            s.live_calls,
            s.work_calls + s.charge_calls + s.check_calls,
            s.changes,
            s.checkpoints
        );
        let mut lists: Vec<_> = s.lists.iter().collect();
        lists.sort_by_key(|(_, c)| std::cmp::Reverse(c.ledger_work + c.ledger_charge + c.ledger_check));
        let _ = writeln!(out, "LISTS name | ops pushes exact_pushes push_bytes pops extends extend_bytes reservations reserved_new_bytes reserved_old_bytes | ledger work charge check changes live");
        for (name, c) in lists {
            let _ = writeln!(
                out,
                "LIST {name} | {} {} {} {} {} {} {} {} {} {} | {} {} {} {} {}",
                c.ops, c.pushes, c.exact_pushes, c.push_bytes, c.pops, c.extends, c.extend_bytes, c.reservations,
                c.reserved_new_bytes, c.reserved_old_bytes, c.ledger_work, c.ledger_charge, c.ledger_check, c.ledger_changes, c.ledger_live
            );
        }
        let mut sites: Vec<_> = s.sites.iter().collect();
        sites.sort_by_key(|(_, c)| std::cmp::Reverse(c.ledger_work + c.ledger_charge + c.ledger_check));
        let _ = writeln!(out, "SITES file:line | work charge check changes");
        for ((file, line), c) in sites.iter() {
            let _ = writeln!(out, "SITE {file}:{line} | {} {} {} {} {}", c.ledger_work, c.ledger_charge, c.ledger_check, c.ledger_changes, c.ledger_live);
        }
        for (k, v) in &s.rows {
            let _ = writeln!(out, "ROW {k} {v}");
        }
        let _ = writeln!(out, "RSITES file:line geometric | reservations new_bytes old_bytes");
        let mut rs: Vec<_> = s.rsites.iter().collect();
        rs.sort_by_key(|(_, c)| std::cmp::Reverse(c.reserved_new_bytes));
        for ((file, line, g), c) in rs {
            let _ = writeln!(out, "RSITE {file}:{line} {g} | {} {} {}", c.reservations, c.reserved_new_bytes, c.reserved_old_bytes);
        }
        out
    })
}
