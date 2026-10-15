//! Actual shared reflected slots, with no CommandFor slot on Input.
use surgeist_html::KnownHtmlAttribute;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReflectedTargets<K> {
    Button {
        command_for: Option<K>,
        popover_target: Option<K>,
    },
    Input {
        popover_target: Option<K>,
    },
}
impl<K: Copy> ReflectedTargets<K> {
    pub(crate) fn button() -> Self {
        Self::Button {
            command_for: None,
            popover_target: None,
        }
    }
    pub(crate) fn input() -> Self {
        Self::Input {
            popover_target: None,
        }
    }
    pub(crate) fn get(self, key: ReflectedTarget) -> Option<Option<K>> {
        match (self, key) {
            (Self::Button { command_for, .. }, ReflectedTarget::CommandFor) => Some(command_for),
            (
                Self::Button { popover_target, .. } | Self::Input { popover_target },
                ReflectedTarget::PopoverTarget,
            ) => Some(popover_target),
            _ => None,
        }
    }
    pub(crate) fn map<T>(self, mut f: impl FnMut(K) -> T) -> ReflectedTargets<T> {
        match self {
            Self::Button {
                command_for,
                popover_target,
            } => ReflectedTargets::Button {
                command_for: command_for.map(&mut f),
                popover_target: popover_target.map(f),
            },
            Self::Input { popover_target } => ReflectedTargets::Input {
                popover_target: popover_target.map(f),
            },
        }
    }
    pub(crate) fn slot_mut(&mut self, key: ReflectedTarget) -> Option<&mut Option<K>> {
        match (self, key) {
            (Self::Button { command_for, .. }, ReflectedTarget::CommandFor) => Some(command_for),
            (
                Self::Button { popover_target, .. } | Self::Input { popover_target },
                ReflectedTarget::PopoverTarget,
            ) => Some(popover_target),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum ReflectedTarget {
    CommandFor,
    PopoverTarget,
}
impl ReflectedTarget {
    pub(crate) fn attribute(self) -> KnownHtmlAttribute {
        match self {
            Self::CommandFor => KnownHtmlAttribute::Commandfor,
            Self::PopoverTarget => KnownHtmlAttribute::Popovertarget,
        }
    }
}
