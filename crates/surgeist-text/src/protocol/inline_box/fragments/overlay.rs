//! The admitted native-only overlay borrows captured parent/event facts.
//! No captured tree, per-box allocation or source transcript is duplicated.
use super::*;
use crate::protocol::inline_box::incidence::SourceIncidence;

impl FamilyData {
    pub(super) fn admit_native_overlay(&mut self, ledger: &mut Ledger) -> Result<()> {
        let mut active: Vec<usize> = Vec::new();
        for index in 0..self.native_boxes.len() {
            account::work(ledger, 1)?;
            let (begin, end) = self.interval(Owner::Native(index));
            while let Some(&other) = active.last() {
                account::work(ledger, 1)?;
                let (_, b) = self.interval(Owner::Native(other));
                if b < begin {
                    active.pop();
                    continue;
                }
                let (start, close) = self.native_boxes[index]
                    .endpoints()
                    .expect("actual native pair");
                let (other_start, other_close) = self.native_boxes[other]
                    .endpoints()
                    .expect("actual native pair");
                // Custody, not a numerical source-cut equality: distinct real
                // controls at the same UTF-16 position remain distinct.
                if start == other_start || close == other_close || b < end {
                    return Err(crate::protocol::inline_box::invalid(
                        crate::InlineBoxEdgeError::InvalidPair,
                    ));
                }
                break;
            }
            let enclosing = active.last().copied();
            let scope = match &self.native_boxes[index].kind {
                InlineBoxSourceKind::NativePair { start, .. } => start.scope().index,
                InlineBoxSourceKind::Captured(_) => {
                    unreachable!("canonical native declarations exclude captured nominations")
                }
            };
            let (p, q) = self.interval(Owner::Scope(scope));
            let parent = match enclosing {
                Some(other) => {
                    let (a, b) = self.interval(Owner::Native(other));
                    if p < a && b < q {
                        Owner::Native(other)
                    } else {
                        Owner::Scope(scope)
                    }
                }
                None => Owner::Scope(scope),
            };
            account::push_fallible(
                ledger,
                &mut self.native_ancestors,
                NativeAncestor { enclosing, parent },
            )?;
            account::push_fallible(ledger, &mut active, index)?;
        }
        account::drop_vec(ledger, active)?;
        if self.native_boxes.is_empty() {
            return Ok(());
        }
        let prepared = self.prepared.inner.clone();
        let topology = prepared
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .expect("admitted captured family");
        for (scope, row) in topology.capture.scopes.iter().enumerate() {
            account::work(ledger, 1)?;
            if row.domain != topology.domain {
                continue;
            }
            let Some(parent) = row
                .parent
                .filter(|&p| topology.capture.scopes[p].domain == topology.domain)
            else {
                continue;
            };
            if let Some(native) = self.native_containing(row.enter, row.exit, ledger)? {
                let (a, b) = self.interval(Owner::Native(native));
                let p = &topology.capture.scopes[parent];
                if p.enter < a && b < p.exit {
                    account::push_fallible(
                        ledger,
                        &mut self.captured_native_parents,
                        (scope, native),
                    )?;
                }
            }
        }
        Ok(())
    }
    /// Existing immutable declaration order is the exact native-source index.
    pub(super) fn owner(&self, source: &InlineBoxSource, ledger: &mut Ledger) -> Result<Owner> {
        let topology = self
            .prepared
            .inner
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .expect("admitted capture");
        match &source.kind {
            InlineBoxSourceKind::Captured(scope)
                if Arc::ptr_eq(&scope.capture, &topology.capture) =>
            {
                Ok(Owner::Scope(scope.index))
            }
            InlineBoxSourceKind::NativePair { start, end }
                if Arc::ptr_eq(&start.capture, &topology.capture)
                    && Arc::ptr_eq(&end.capture, &topology.capture) =>
            {
                let key = (start.event_position().index, end.event_position().index);
                let (mut a, mut b) = (0, self.native_boxes.len());
                while a < b {
                    account::work(ledger, 1)?;
                    let m = a + (b - a) / 2;
                    if self.interval(Owner::Native(m)) < key {
                        a = m + 1
                    } else {
                        b = m
                    }
                }
                if self.native_boxes.get(a).is_some_and(|v| v == source) {
                    Ok(Owner::Native(a))
                } else {
                    Err(invalid(InlineBoxFragmentationError::ForeignFamily))
                }
            }
            _ => Err(invalid(InlineBoxFragmentationError::ForeignFamily)),
        }
    }
    pub(crate) fn parent(&self, owner: Owner, ledger: &mut Ledger) -> Result<Option<Owner>> {
        account::work(ledger, 1)?;
        match owner {
            Owner::Native(index) => Ok(Some(self.native_ancestors[index].parent)),
            Owner::Scope(index) => {
                let (mut a, mut b) = (0, self.captured_native_parents.len());
                while a < b {
                    account::work(ledger, 1)?;
                    let m = a + (b - a) / 2;
                    if self.captured_native_parents[m].0 < index {
                        a = m + 1
                    } else {
                        b = m
                    }
                }
                if let Some(&(scope, native)) = self.captured_native_parents.get(a)
                    && scope == index
                {
                    return Ok(Some(Owner::Native(native)));
                }
                let topology = self
                    .prepared
                    .inner
                    .analyzed
                    .paragraph
                    .scope_topology
                    .as_ref()
                    .expect("admitted capture");
                Ok(topology.capture.scopes[index]
                    .parent
                    .filter(|&p| topology.capture.scopes[p].domain == topology.domain)
                    .map(Owner::Scope))
            }
        }
    }
    fn preceding_native(&self, position: usize, ledger: &mut Ledger) -> Result<Option<usize>> {
        let (mut a, mut b) = (0, self.native_boxes.len());
        while a < b {
            account::work(ledger, 1)?;
            let m = a + (b - a) / 2;
            if self.interval(Owner::Native(m)).0 < position {
                a = m + 1
            } else {
                b = m
            }
        }
        Ok(a.checked_sub(1))
    }
    fn native_containing(
        &self,
        begin: usize,
        end: usize,
        ledger: &mut Ledger,
    ) -> Result<Option<usize>> {
        let mut at = self.preceding_native(begin, ledger)?;
        while let Some(index) = at {
            account::work(ledger, 1)?;
            let (_, b) = self.interval(Owner::Native(index));
            if end < b {
                return Ok(Some(index));
            }
            at = self.native_ancestors[index].enclosing;
        }
        Ok(None)
    }
    pub(super) fn native_at(
        &self,
        site: SourceIncidence,
        ledger: &mut Ledger,
    ) -> Result<Option<usize>> {
        let mut at = self.preceding_native(site.position(), ledger)?;
        while let Some(index) = at {
            account::work(ledger, 1)?;
            let (_, end) = self.interval(Owner::Native(index));
            if site.before_end(end) {
                return Ok(Some(index));
            }
            at = self.native_ancestors[index].enclosing;
        }
        Ok(None)
    }
}
