//! A table of the rows one family keeps for the nodes that have them.
//!
//! A node's position is its creation order, so a row pushed in the step that creates its node is
//! pushed in ascending order of key; a lookup is a binary search over the keys. A node that has no
//! row is never searched for: the family of an element is decided before its row is asked for.
//! Admission and the write are separate steps, as for every other bank of the parse: [`admit`]
//! reserves both vectors and acquires the entries of the rows to come, and [`push`] only writes
//! into what was admitted.
//!
//! [`admit`]: FamilyTable::admit
//! [`push`]: FamilyTable::push

use super::session_error::latch_registry_row;
use super::*;
use crate::parser::control::VectorCapacity;

pub(super) struct FamilyTable<T> {
    nodes: Vec<u32>,
    nodes_allocation: VectorCapacity<u32>,
    rows: Vec<T>,
    rows_allocation: VectorCapacity<T>,
    /// The rows [`Self::admit`] has acquired entries for and [`Self::push`] has not written.
    admitted: usize,
}

impl<T> FamilyTable<T> {
    pub(super) fn new(control: &mut ParseControl) -> Result<Self, SessionError> {
        let (nodes, nodes_allocation) = control.new_vector()?;
        let (rows, rows_allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                release_vector(control, nodes, nodes_allocation)?;
                return Err(stop.into());
            }
        };
        Ok(Self {
            nodes,
            nodes_allocation,
            rows,
            rows_allocation,
            admitted: 0,
        })
    }
    /// Admits `additional` rows: both vectors have room for them and for every row admitted and not
    /// yet written, and their auxiliary entries are acquired. A refusal leaves nothing admitted
    /// that this call added.
    #[cfg_attr(test, track_caller)]
    pub(super) fn admit(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), SessionError> {
        let admitted = self
            .admitted
            .checked_add(additional)
            .ok_or_else(|| latch_registry_row(control))?;
        // Rows admitted and not yet written take room beyond the written length too.
        control.reserve_vector_geometric(&mut self.nodes, &mut self.nodes_allocation, admitted)?;
        control.reserve_vector_geometric(&mut self.rows, &mut self.rows_allocation, admitted)?;
        control.admit_atomic_layout(mem::size_of::<T>())?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            (mem::size_of::<T>() as u64).saturating_mul(additional as u64),
        )?;
        control.charge(HtmlParseWorkCategory::RecordWrite, additional as u64)?;
        control.acquire(HtmlParseResource::AuxiliaryEntries, additional as u64)?;
        self.admitted = admitted;
        Ok(())
    }
    /// Writes the row of `node`, which must be above every key already in the table, into
    /// admitted room. A node that is not above the last key, a row that was not admitted and a
    /// node position the key cannot hold are the registry-row defect; the table is unchanged.
    pub(super) fn push(&mut self, node: usize, row: T) -> Result<(), SessionError> {
        let key = u32::try_from(node).map_err(|_| SessionError::registry_row())?;
        if self.nodes.last().is_some_and(|&last| last >= key)
            || self.admitted == 0
            || self.nodes.len() >= self.nodes.capacity()
            || self.rows.len() >= self.rows.capacity()
        {
            return Err(SessionError::registry_row());
        }
        self.admitted = self
            .admitted
            .checked_sub(1)
            .ok_or_else(SessionError::registry_row)?;
        self.nodes.push(key);
        self.rows.push(row);
        Ok(())
    }
    /// The index of the row of `node`, if it has one. The search is a binary search, and one
    /// record visit is charged before each key it reads: a table of `n` keys costs at most
    /// `floor(log2(n)) + 1`, and a table that holds one key costs one. A node the key cannot hold
    /// is in no table and costs nothing.
    pub(super) fn find(
        &self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<Option<usize>, SessionError> {
        let Ok(key) = u32::try_from(node) else {
            return Ok(None);
        };
        let mut low = 0_usize;
        let mut high = self.nodes.len();
        while low < high {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let middle = low.saturating_add(high.saturating_sub(low) / 2);
            let held = *self
                .nodes
                .get(middle)
                .ok_or_else(|| latch_registry_row(control))?;
            match held.cmp(&key) {
                std::cmp::Ordering::Equal => return Ok(Some(middle)),
                std::cmp::Ordering::Less => low = middle.saturating_add(1),
                std::cmp::Ordering::Greater => high = middle,
            }
        }
        Ok(None)
    }
    /// The row of `node`, if it has one, charged as [`Self::find`].
    pub(super) fn get(
        &self,
        control: &mut ParseControl,
        node: usize,
    ) -> Result<Option<&T>, SessionError> {
        let Some(index) = self.find(control, node)? else {
            return Ok(None);
        };
        self.rows
            .get(index)
            .map(Some)
            .ok_or_else(|| latch_registry_row(control))
    }
    /// The node that owns the row at `index`.
    pub(super) fn node_at(&self, index: usize) -> Option<usize> {
        usize::try_from(*self.nodes.get(index)?).ok()
    }
    /// The row at `index`, an index [`Self::find`] returned.
    pub(super) fn row(&self, index: usize) -> Option<&T> {
        self.rows.get(index)
    }
    pub(super) fn row_mut(&mut self, index: usize) -> Option<&mut T> {
        self.rows.get_mut(index)
    }
    /// The rows with the nodes that own them, in ascending order of node.
    pub(super) fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.nodes
            .iter()
            .zip(&self.rows)
            .filter_map(|(&node, row)| Some((usize::try_from(node).ok()?, row)))
    }
    pub(super) fn len(&self) -> usize {
        self.rows.len()
    }
    /// Releases both banks and every entry of the table, rows admitted and not written included.
    /// `dispose` is called with each row before it is dropped, for a row that owns a ticket.
    pub(super) fn release(
        self,
        control: &mut ParseControl,
        owner: &ControlIdentity,
        mut dispose: impl FnMut(&mut ParseControl, &mut T) -> Result<(), SessionError>,
    ) -> Result<(), SessionError> {
        let Self {
            nodes,
            nodes_allocation,
            mut rows,
            rows_allocation,
            admitted,
        } = self;
        let entries = (rows.len() as u64).saturating_add(admitted as u64);
        let mut refused = None;
        for row in &mut rows {
            keep_first(&mut refused, dispose(control, row));
        }
        keep_first(
            &mut refused,
            release_vector(control, nodes, nodes_allocation),
        );
        keep_first(&mut refused, release_vector(control, rows, rows_allocation));
        keep_first(
            &mut refused,
            release_logical(control, owner, HtmlParseResource::AuxiliaryEntries, entries),
        );
        refused.map_or(Ok(()), Err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control() -> ParseControl {
        ParseControl::new(
            super::super::tests::limits(),
            crate::HtmlParseCancellation::new(),
        )
    }

    fn entries(c: &ParseControl) -> u64 {
        c.usage()
            .resource(HtmlParseResource::AuxiliaryEntries)
            .current
    }

    #[test]
    fn a_row_vector_shorter_than_its_keys_latches_the_registry_row_stop() {
        let mut c = control();
        let mut table: FamilyTable<u64> = FamilyTable::new(&mut c).unwrap();
        table.admit(&mut c, 2).unwrap();
        table.push(4, 40).unwrap();
        table.push(9, 90).unwrap();
        // A key whose row is missing is a state `push` cannot make.
        table.rows.truncate(1);
        let stop = HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::RegistryRow,
        );
        assert_eq!(table.get(&mut c, 4), Ok(Some(&40)));
        assert_eq!(table.get(&mut c, 9), Err(SessionError::Stop(stop)));
        assert_eq!(c.stop(), Some(stop));
    }

    #[test]
    fn rows_are_found_by_node_at_every_position() {
        let mut c = control();
        let owner = c.identity();
        let mut table: FamilyTable<u64> = FamilyTable::new(&mut c).unwrap();
        let nodes = [2_usize, 3, 7, 40, 41, 1_000];
        table.admit(&mut c, nodes.len()).unwrap();
        for (index, &node) in nodes.iter().enumerate() {
            table.push(node, index as u64 * 10).unwrap();
        }
        assert_eq!(table.len(), nodes.len());
        for probe in 0..1_100_usize {
            let expected = nodes.iter().position(|&node| node == probe);
            assert_eq!(table.find(&mut c, probe).unwrap(), expected, "node {probe}");
            assert_eq!(
                table.get(&mut c, probe).unwrap().copied(),
                expected.map(|index| index as u64 * 10)
            );
        }
        assert_eq!(table.find(&mut c, usize::MAX).unwrap(), None);
        let walked: Vec<usize> = table.iter().map(|(node, _)| node).collect();
        assert_eq!(walked, nodes);
        table.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn a_key_that_is_not_above_the_last_is_refused_and_the_table_is_unchanged() {
        let mut c = control();
        let owner = c.identity();
        let mut table: FamilyTable<u8> = FamilyTable::new(&mut c).unwrap();
        table.admit(&mut c, 3).unwrap();
        table.push(5, 1).unwrap();
        for node in [5, 4, 0] {
            assert_eq!(table.push(node, 9), Err(SessionError::registry_row()));
            assert_eq!(table.len(), 1);
            assert_eq!(table.get(&mut c, 5).unwrap(), Some(&1));
            assert_eq!(
                table.get(&mut c, node).unwrap(),
                if node == 5 { Some(&1) } else { None }
            );
        }
        table.push(6, 2).unwrap();
        assert_eq!(table.get(&mut c, 6).unwrap(), Some(&2));
        table.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn a_row_that_was_not_admitted_is_refused() {
        let mut c = control();
        let owner = c.identity();
        let mut table: FamilyTable<u8> = FamilyTable::new(&mut c).unwrap();
        assert_eq!(table.push(1, 1), Err(SessionError::registry_row()));
        table.admit(&mut c, 1).unwrap();
        table.push(1, 1).unwrap();
        assert_eq!(table.push(2, 2), Err(SessionError::registry_row()));
        assert_eq!(table.len(), 1);
        table.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn a_node_position_the_key_cannot_hold_is_refused() {
        let mut c = control();
        let owner = c.identity();
        let mut table: FamilyTable<u8> = FamilyTable::new(&mut c).unwrap();
        table.admit(&mut c, 1).unwrap();
        assert_eq!(
            table.push(usize::try_from(u64::from(u32::MAX) + 1).unwrap(), 1),
            Err(SessionError::registry_row())
        );
        assert_eq!(table.len(), 0);
        table.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn admission_acquires_one_entry_per_row_and_release_returns_every_one() {
        let mut c = control();
        let owner = c.identity();
        let base = entries(&c);
        let mut table: FamilyTable<u32> = FamilyTable::new(&mut c).unwrap();
        table.admit(&mut c, 2).unwrap();
        assert_eq!(entries(&c), base + 2);
        table.push(1, 1).unwrap();
        // One admitted row is left unwritten; its entry is returned with the rest.
        table.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        assert_eq!(entries(&c), base);
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn two_admissions_before_their_writes_leave_room_for_both_rows() {
        let mut c = control();
        let owner = c.identity();
        let base = entries(&c);
        let mut table: FamilyTable<u32> = FamilyTable::new(&mut c).unwrap();
        table.admit(&mut c, 1).unwrap();
        table.admit(&mut c, 1).unwrap();
        assert_eq!(entries(&c), base + 2);
        table.push(1, 1).unwrap();
        table.push(2, 2).unwrap();
        assert_eq!(table.len(), 2);
        assert_eq!(table.get(&mut c, 2).unwrap(), Some(&2));
        table.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        assert_eq!(entries(&c), base);
        super::super::tests::storage_zero(&c);
    }

    #[test]
    fn a_refused_bank_reservation_keeps_the_reserved_vector_accounted_and_admits_nothing() {
        // The key bank takes 4 bytes for one row; the row bank takes 64 and is refused.
        let limits = super::super::tests::limits()
            .into_builder()
            .managed_capacity_bytes(4 + 63)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        let owner = c.identity();
        let base = entries(&c);
        let capacity = |c: &ParseControl| {
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
        };
        let mut table: FamilyTable<[u8; 64]> = FamilyTable::new(&mut c).unwrap();
        assert_eq!(capacity(&c), 0);
        assert!(matches!(
            table.admit(&mut c, 1),
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::ManagedCapacityBytes,
                ..
            }))
        ));
        assert_eq!(capacity(&c), 4);
        assert_eq!(table.admitted, 0);
        assert_eq!(entries(&c), base);
        assert_eq!(table.push(1, [0; 64]), Err(SessionError::registry_row()));
        assert_eq!(table.len(), 0);
        let stop = c.stop();
        table.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        assert_eq!(capacity(&c), 0);
        assert_eq!(entries(&c), base);
        super::super::tests::storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }

    #[test]
    fn a_refused_admission_leaves_the_table_as_it_was() {
        let base = entries(&control());
        let limits = super::super::tests::limits()
            .into_builder()
            .auxiliary_entries(base + 1)
            .active_formatting_entries(base + 1)
            .finish()
            .unwrap();
        let mut c = ParseControl::new(limits, crate::HtmlParseCancellation::new());
        let owner = c.identity();
        let mut table: FamilyTable<u32> = FamilyTable::new(&mut c).unwrap();
        table.admit(&mut c, 1).unwrap();
        table.push(1, 1).unwrap();
        assert!(matches!(
            table.admit(&mut c, 1),
            Err(SessionError::Stop(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::AuxiliaryEntries,
                ..
            }))
        ));
        assert_eq!(table.len(), 1);
        assert_eq!(table.push(2, 2), Err(SessionError::registry_row()));
        let stop = c.stop();
        table.release(&mut c, &owner, |_, _| Ok(())).unwrap();
        super::super::tests::storage_zero(&c);
        assert_eq!(c.stop(), stop);
    }

    /// A table of `n = 2^m - 1` keys is a complete binary tree of depth `m`: one key is found at
    /// the first probe, two at the second, and `2^(p - 1)` at probe `p`, so the keys cost
    /// `1 + 2 * 2 + ... + 2^(m - 1) * m = (m - 1) * 2^m + 1` probes in all. A key above every
    /// held key is decided after `m` probes.
    #[test]
    fn a_lookup_of_a_document_charges_the_probes_of_the_search() {
        for m in 1..=6_u32 {
            let keys = (1_usize << m) - 1;
            let mut c = control();
            let mut session = ParserArenaSession::new(
                &mut c,
                RootKind::Document(surgeist_html::construction::HtmlDocumentMode::NoQuirks),
            )
            .unwrap();
            {
                let documents = &mut session.active_mut().unwrap().payloads.facts.documents;
                documents.admit(&mut c, keys - 1).unwrap();
                for key in 1..keys {
                    documents
                        .push(
                            key,
                            template::DocumentFacts {
                                state: template::DocumentState::InertTemplate,
                                class: crate::HtmlDocumentClass::Html,
                                meta: meta::MetaHistory::offline(),
                                interaction: dialog::InteractionHistory::new(),
                            },
                        )
                        .unwrap();
                }
            }
            let visits = |c: &ParseControl| c.usage().work(HtmlParseWorkCategory::RecordVisit);
            let mut total = 0;
            let mut most = 0;
            let storage = session.active().unwrap();
            for key in 0..keys {
                let before = visits(&c);
                storage.csp_policy_slot(&mut c, key).unwrap();
                let probes = visits(&c) - before;
                total += probes;
                most = most.max(probes);
            }
            assert_eq!(total, u64::from(m - 1) * (1_u64 << m) + 1, "{keys} keys");
            assert_eq!(most, u64::from(m), "{keys} keys");
            let before = visits(&c);
            storage.csp_policy_slot(&mut c, keys + 5).unwrap();
            assert_eq!(visits(&c) - before, u64::from(m), "{keys} keys, absent");
            session.release(&mut c).unwrap();
            super::super::tests::storage_zero(&c);
        }
    }
}
