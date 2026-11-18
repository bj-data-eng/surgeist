use super::*;

const DEFAULT_PASSIVE_TYPES: [&[u16]; 4] = [
    &[116, 111, 117, 99, 104, 115, 116, 97, 114, 116], // touchstart
    &[116, 111, 117, 99, 104, 109, 111, 118, 101],     // touchmove
    &[119, 104, 101, 101, 108],                        // wheel
    &[109, 111, 117, 115, 101, 119, 104, 101, 101, 108], // mousewheel
];

impl<C> DomEventDispatcher<C> {
    pub(super) fn default_passive(
        &self,
        dom: &Dom,
        target: &DomEventTarget,
        event_type: &[u16],
        cancel: &AtomicBool,
    ) -> Result<bool, DomEventDispatchError> {
        for expected in DEFAULT_PASSIVE_TYPES {
            if expected.len() == event_type.len()
                && equal_units(expected, event_type, || self.storage.step(1, cancel))?
            {
                return match target {
                    DomEventTarget::Node(node) => {
                        dom.is_default_passive_listener_node(*node, cancel, &mut || {
                            self.storage.step(1, cancel)
                        })
                    }
                    // Registration already qualified the native Window association.
                    DomEventTarget::Global(_) => Ok(true),
                    DomEventTarget::Standalone(_) => Ok(false),
                };
            }
        }
        Ok(false)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum PassiveScope {
    None,
    Borrowed,
    Host,
}

/// A borrowed passive callback retains its flag through actual stack unwind
/// and through registry abandonment initiated by the callback itself.
pub(super) struct BorrowedPassive<'a>(&'a Cell<PassiveScope>);

impl<'a> BorrowedPassive<'a> {
    pub(super) fn begin(scope: &'a Cell<PassiveScope>, passive: bool) -> Option<Self> {
        passive.then(|| {
            scope.set(PassiveScope::Borrowed);
            Self(scope)
        })
    }
}

impl Drop for BorrowedPassive<'_> {
    fn drop(&mut self) {
        self.0.set(PassiveScope::None);
    }
}
