use super::*;

const DIGIT_BITS: u32 = 4;
const RADIX: usize = 1 << DIGIT_BITS;
const DIGIT_MASK: u64 = (1 << DIGIT_BITS) - 1;

impl Dom {
    /// Resolve a zero-based Window index to its current native child browsing context.
    ///
    /// The Window must have an active associated Document. Its ordinary-tree
    /// children are ordered by structural insertion group, with current tree
    /// order for ties. An index beyond those children returns `None`.
    ///
    /// # Errors
    ///
    /// Returns the first invalid hosted identity, Source or caller refusal,
    /// cancellation, or exhausted work. All passes share one work allowance
    /// and the exact active Source scope.
    pub fn window_indexed_child<E: From<DomHostedError>>(
        &self,
        global: DomHostedGlobalId,
        index: usize,
        work_limit: u64,
        mut admission: impl FnMut() -> Result<(), E>,
        cancel: &AtomicBool,
    ) -> Result<Option<DomHostedBrowsingContextId>, E> {
        if global.source.owner != self.owner {
            return Err(DomHostedError::ForeignSource.into());
        }
        let budget =
            HostedReadBudget::new(self, work_limit, cancel).map_err(DomHostedError::from)?;
        let mut read = || -> Result<_, HostedReadError<E>> {
            let mut step = || budget.step(&mut admission);
            let (document, metadata, _) =
                self.dispatch_global_document_metadata_with(global, &mut step)?;
            let children = self
                .qualify_document_tree_children(document, metadata, &mut step)?
                .ok_or(DomHostedError::StaleIdentity)?;
            children.indexed(&self.snapshot(), index, &mut step)
        };
        read().map_err(HostedReadError::into_caller)
    }
}

impl DocumentTreeChildren<'_> {
    fn indexed<E, S>(
        &self,
        snapshot: &DomSnapshot,
        index: usize,
        step: &mut S,
    ) -> Result<Option<DomHostedBrowsingContextId>, E>
    where
        E: From<DomError> + From<DomHostedError>,
        S: FnMut() -> Result<(), E>,
    {
        let mut count = 0usize;
        let mut maximum_group = 0;
        let mut earliest = None;
        let mut tree_ordinal = None;
        self.find(snapshot, step, |child, _, step| {
            step()?;
            let group = insertion_group(child)?;
            let context = child.browsing_context();
            if count == index {
                tree_ordinal = Some(context);
            }
            count = count
                .checked_add(1)
                .ok_or(DomHostedError::RepresentationExhausted)?;
            maximum_group = maximum_group.max(group);
            if earliest.is_none_or(|(minimum, _)| group < minimum) {
                earliest = Some((group, context));
            }
            // Group zero cannot have an earlier insertion group.
            Ok(index == 0 && group == 0)
        })?;
        if index >= count {
            return Ok(None);
        }
        let (minimum_group, first) = earliest.ok_or(DomHostedError::StaleBasis)?;
        if index == 0 {
            return Ok(Some(first));
        }
        if minimum_group == maximum_group {
            return tree_ordinal
                .map(Some)
                .ok_or_else(|| DomHostedError::StaleBasis.into());
        }

        // Only differing digits can distinguish groups. Each pass keeps the
        // selected prefix and rank within that prefix without retaining children.
        let differing_bits = u64::BITS - (minimum_group ^ maximum_group).leading_zeros();
        let first_shift = differing_bits.saturating_sub(1) / DIGIT_BITS * DIGIT_BITS;
        let mut prefix = (minimum_group >> first_shift) & !DIGIT_MASK;
        let mut rank = index;
        for shift in (0..=first_shift).rev().step_by(4) {
            let mut histogram = [0usize; RADIX];
            self.find(snapshot, step, |child, _, step| {
                step()?;
                let digits = insertion_group(child)? >> shift;
                if digits & !DIGIT_MASK == prefix {
                    let digit = usize::try_from(digits & DIGIT_MASK)
                        .map_err(|_| DomHostedError::RepresentationExhausted)?;
                    let bucket = histogram
                        .get_mut(digit)
                        .ok_or(DomHostedError::RepresentationExhausted)?;
                    *bucket = bucket
                        .checked_add(1)
                        .ok_or(DomHostedError::RepresentationExhausted)?;
                }
                Ok(false)
            })?;
            let mut selected = None;
            for (digit, count) in histogram.into_iter().enumerate() {
                step()?;
                match rank.checked_sub(count) {
                    Some(remaining) => rank = remaining,
                    None => {
                        selected = Some(
                            u64::try_from(digit)
                                .map_err(|_| DomHostedError::RepresentationExhausted)?,
                        );
                        break;
                    }
                }
            }
            prefix |= selected.ok_or(DomHostedError::StaleBasis)?;
            if shift != 0 {
                prefix <<= DIGIT_BITS;
            }
        }

        let mut context = None;
        self.find(snapshot, step, |child, _, step| {
            step()?;
            if insertion_group(child)? != prefix {
                return Ok(false);
            }
            match rank.checked_sub(1) {
                Some(remaining) => {
                    rank = remaining;
                    Ok(false)
                }
                None => {
                    context = Some(child.browsing_context());
                    Ok(true)
                }
            }
        })?;
        context
            .map(Some)
            .ok_or_else(|| DomHostedError::StaleBasis.into())
    }
}

fn insertion_group(child: DomHostedNavigable<'_>) -> Result<u64, DomHostedError> {
    child
        .container_insertion()
        .map(DomHostedChildInsertion::group)
        .ok_or(DomHostedError::MissingPublication)
}
