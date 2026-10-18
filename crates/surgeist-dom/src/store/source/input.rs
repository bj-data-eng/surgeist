//! Owner-only custody of an original whole-snapshot command capture. Descendant
//! import frames alias this capsule; semantic snapshots themselves carry none.
use super::*;

#[derive(Clone)]
pub(in crate::store) struct InputCapture {
    _state: Arc<State>,
    _custody: transaction::StateCustody,
}
const PART_BITS: usize = usize::BITS as usize - 1;
const LAST_PART: usize = 64_usize.div_ceil(PART_BITS) - 1;
fn part(identity: u64, position: usize) -> usize {
    ((identity >> (position * PART_BITS)) as usize) & ((1_usize << PART_BITS) - 1)
}
pub(in crate::store) enum InputEntry {
    Branch {
        values: bank::Bank<InputEntry>,
        custody: bank::BankCustody<InputEntry>,
    },
    Capture(InputCapture),
}
fn insert(
    values: &bank::Bank<InputEntry>,
    custody: &bank::BankCustody<InputEntry>,
    identity: u64,
    position: usize,
    capture: InputCapture,
    registry: &bank::CustodyRegistry,
    admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<(bank::Bank<InputEntry>, bank::BankCustody<InputEntry>), DomError> {
    let key = part(identity, position);
    let entry = if position == 0 {
        InputEntry::Capture(capture)
    } else {
        let (children, control) = match values.get(key) {
            Some(InputEntry::Branch { values, custody }) => (values.clone(), custody.clone()),
            None => (bank::Bank::default(), bank::BankCustody::default()),
            Some(InputEntry::Capture(_)) => unreachable!("input identity branch depth"),
        };
        let (values, custody) = insert(
            &children,
            &control,
            identity,
            position - 1,
            capture,
            registry,
            admit,
        )?;
        InputEntry::Branch { values, custody }
    };
    let usage = custody::Usage::new(size_of::<InputEntry>(), 0);
    let record = bank::ControlledRecord::new(entry, usage, registry, &mut *admit)?;
    values.set_controlled(
        custody,
        key,
        record,
        false,
        bank::Totals {
            bytes: usage.bytes(),
            units: 0,
            occurrences: 1,
        },
        registry,
        admit,
    )
}
impl Draft {
    pub(in crate::store) fn retain_input_capture(
        &mut self,
        state: &Arc<State>,
        cancel: &AtomicBool,
    ) -> Result<InputCapture, DomError> {
        let identity = state.identity.key();
        let mut current = &self.input_captures;
        for position in (0..=LAST_PART).rev() {
            match current.get(part(identity, position)) {
                Some(InputEntry::Branch { values, .. }) => current = values,
                Some(InputEntry::Capture(capture)) => return Ok(capture.clone()),
                None => break,
            }
        }
        let custody = transaction::StateCustody::adopt(
            state,
            self.custody_registry(),
            |usage| self.admit_custody_usage(usage, cancel),
            || self.source_step(cancel),
        )?;
        let capture = InputCapture {
            _state: state.clone(),
            _custody: custody,
        };
        let (values, custody) = insert(
            &self.input_captures,
            &self.input_captures_custody,
            identity,
            LAST_PART,
            capture.clone(),
            self.custody_registry(),
            &mut |usage| self.admit_custody_usage(usage, cancel),
        )?;
        self.input_captures = values;
        self.input_captures_custody = custody;
        Ok(capture)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_identity_parts_preserve_every_bit_and_leave_index_extent_room() {
        for identity in [0, 1, 1_u64 << 31, 1_u64 << 32, 1_u64 << 63, u64::MAX] {
            let mut restored = 0_u64;
            for position in 0..=LAST_PART {
                let value = part(identity, position);
                assert!(value < usize::MAX);
                restored |= (value as u64) << (position * PART_BITS);
            }
            assert_eq!(restored, identity);
        }
        // The 32-bit representation has literal 31-bit branches and retains
        // the final two high bits; this does not rely on pointer truncation.
        let identity = 0xfedc_ba98_7654_3210_u64;
        assert_eq!(identity & 0x7fff_ffff, 0x7654_3210);
        assert_eq!((identity >> 31) & 0x7fff_ffff, 0x7db9_7530);
        assert_eq!(identity >> 62, 3);
    }
}
