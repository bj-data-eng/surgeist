//! Admitted dense-locator compaction for actual retired inline bindings.
use super::*;

pub(in crate::store) struct RetirementPlan {
    swaps: Vec<(usize, usize)>,
    len: usize,
    _debit: custody::Debit,
}
impl RetirementPlan {
    pub(in crate::store) fn publish(self, objects: &mut Vec<ObjectEntry>) {
        for (hole, tail) in self.swaps {
            objects.swap(hole, tail);
        }
        objects.truncate(self.len);
    }
}
impl Owner {
    pub(super) fn prepare_retirement(
        &self,
        recording: &recording::Recording,
        candidate: &mut Index,
        registry: &bank::CustodyRegistry,
        p: &Processing<'_>,
    ) -> Result<Option<RetirementPlan>, DomError> {
        let retired = recording.retirement_count();
        if retired == 0 {
            return Ok(None);
        }
        let len = self
            .objects
            .len()
            .checked_sub(retired)
            .expect("one retirement per current exact binding");
        let requested = retired
            .checked_mul(size_of::<(usize, usize)>())
            .ok_or(DomError::RepresentationExhausted)?;
        p.scratch(requested)?;
        let mut debit = registry
            .ledger()
            .retain(custody::Usage::new(requested, 0), |u| p.admit(u))?;
        let mut swaps = Vec::new();
        swaps
            .try_reserve_exact(retired)
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = swaps
            .capacity()
            .checked_mul(size_of::<(usize, usize)>())
            .ok_or(DomError::RepresentationExhausted)?;
        p.scratch(actual)?;
        transaction::nodes::resize_debit(&mut debit, custody::Usage::new(actual, 0), |u| {
            p.admit(u)
        })?;
        for index in 0..retired {
            let element = recording.retirement_at(index, p)?;
            let binding = self
                .index
                .membership
                .bindings
                .get_with(element.slot, &mut || p.step())?
                .filter(|binding| binding.element == element)
                .expect("retirement has original exact binding");
            if binding.ordinal < len {
                swaps.push((binding.ordinal, usize::MAX));
            }
        }
        let mut holes = swaps.iter_mut();
        // This tail has exactly r positions. No search over unrelated bindings,
        // vacant object slots or the surviving dense prefix is needed.
        for tail in len..self.objects.len() {
            p.step()?;
            let element = self.objects[tail].element;
            if let Some(binding) = candidate
                .membership
                .bindings
                .get_with(element.slot, &mut || p.step())?
                .filter(|binding| binding.element == element)
            {
                assert_eq!(binding.ordinal, tail);
                let (hole, source) = holes.next().expect("surviving tail fills one retired hole");
                *source = tail;
                candidate.replace_binding(
                    Binding {
                        element,
                        ordinal: *hole,
                    },
                    registry,
                    p,
                )?;
            }
        }
        assert!(
            holes.next().is_none(),
            "every retained prefix hole is filled"
        );
        Ok(Some(RetirementPlan {
            swaps,
            len,
            _debit: debit,
        }))
    }
}
