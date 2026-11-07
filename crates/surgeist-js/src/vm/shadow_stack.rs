use std::fmt::{self, Display};

use surgeist_js_string::JsString;
use thin_vec::ThinVec;

use super::source_info::{NativeSourceInfo, SourceInfo};

#[derive(Debug, Default, Clone)]
pub(crate) struct Backtrace {
    stack: ThinVec<ShadowEntry>,
}

impl Backtrace {
    pub(crate) fn iter(&self) -> impl DoubleEndedIterator<Item = &ShadowEntry> {
        self.stack.iter()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ErrorStack {
    Position(ShadowEntry),
    Backtrace(Backtrace),
}

impl ErrorStack {
    pub(crate) fn backtrace(&self) -> Option<&Backtrace> {
        match self {
            Self::Backtrace(bt) => Some(bt),
            Self::Position(_) => None,
        }
    }

    pub(crate) fn position(&self) -> Option<&ShadowEntry> {
        match self {
            Self::Position(position) => Some(position),
            Self::Backtrace(bt) => bt.iter().next(),
        }
    }

    pub(crate) fn is_backtrace(&self) -> bool {
        matches!(self, Self::Backtrace(_))
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ShadowEntry {
    Native {
        function_name: Option<JsString>,
        source_info: NativeSourceInfo,
    },
    Bytecode {
        pc: u32,
        source_info: SourceInfo,
    },
}

impl ShadowEntry {
    /// Create a display wrapper for this entry.
    ///
    /// # Arguments
    ///
    /// * `show_function_name` - Whether to include the function name in the output.
    pub(crate) fn display(&self, show_function_name: bool) -> DisplayShadowEntry<'_> {
        DisplayShadowEntry {
            entry: self,
            show_function_name,
        }
    }
}

/// Helper struct to format a shadow entry for display.
pub(crate) struct DisplayShadowEntry<'a> {
    entry: &'a ShadowEntry,
    show_function_name: bool,
}

impl Display for DisplayShadowEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.entry {
            ShadowEntry::Native {
                function_name,
                source_info,
            } => {
                if self.show_function_name {
                    if let Some(function_name) = function_name {
                        write!(f, "{}", function_name.to_std_string_escaped())?;
                    } else {
                        f.write_str("<anonymous>")?;
                    }
                }

                if let Some(loc) = source_info.as_location() {
                    write!(
                        f,
                        " (native at {}:{}:{})",
                        loc.file(),
                        loc.line(),
                        loc.column()
                    )?;
                } else {
                    f.write_str(" (native)")?;
                }
            }
            ShadowEntry::Bytecode { pc, source_info } => {
                if self.show_function_name {
                    let has_function_name = !source_info.function_name().is_empty();
                    if has_function_name {
                        write!(f, "{}", source_info.function_name().to_std_string_escaped())?;
                    } else {
                        f.write_str("<main>")?;
                    }
                }
                f.write_str(" (")?;

                source_info.map().path().fmt(f)?;

                if let Some(position) = source_info.map().find(*pc) {
                    write!(
                        f,
                        ":{}:{}",
                        position.line_number(),
                        position.column_number()
                    )?;
                } else {
                    f.write_str(":?:?")?;
                }
                f.write_str(")")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ShadowStack {
    stack: ThinVec<ShadowEntry>,
}

/// One invocation's append boundary and the caller entry its pushes may patch.
/// Source information is shared; this never clones the caller's whole stack.
#[derive(Debug)]
pub(super) struct ShadowCheckpoint {
    len: usize,
    last: Option<ShadowEntry>,
}

impl ShadowStack {
    pub(super) fn checkpoint(&self) -> ShadowCheckpoint {
        ShadowCheckpoint {
            len: self.stack.len(),
            last: self.stack.last().cloned(),
        }
    }

    /// Discards appended entries and undoes the caller patch made by a push.
    /// The invocation must retain the caller prefix represented by this checkpoint.
    pub(super) fn restore(&mut self, checkpoint: ShadowCheckpoint) {
        debug_assert!(self.stack.len() >= checkpoint.len);
        self.stack.truncate(checkpoint.len);
        if let Some(last) = checkpoint.last {
            *self
                .stack
                .last_mut()
                .expect("the invocation retains its caller shadow entry") = last;
        }
    }

    pub(crate) fn push_native(
        &mut self,
        last_pc: u32,
        function_name: JsString,
        native_source_info: NativeSourceInfo,
    ) {
        // NOTE: pc points to the next opcode, so we offset by -1 to put it within range.
        let last_pc = last_pc.saturating_sub(1);

        match self.stack.last_mut() {
            Some(ShadowEntry::Bytecode { pc, .. }) => *pc = last_pc,
            Some(ShadowEntry::Native { source_info, .. }) => *source_info = native_source_info,
            _ => {}
        }
        self.stack.push(ShadowEntry::Native {
            function_name: Some(function_name),
            source_info: native_source_info,
        });
    }

    pub(crate) fn push_bytecode(&mut self, last_pc: u32, source_info: SourceInfo) {
        // NOTE: pc points to the next opcode, so we offset by -1 to put it within range.
        let last_pc = last_pc.saturating_sub(1);

        if let Some(ShadowEntry::Bytecode { pc, .. }) = self.stack.last_mut() {
            *pc = last_pc;
        }
        self.stack
            .push(ShadowEntry::Bytecode { pc: 0, source_info });
    }

    pub(crate) fn pop(&mut self) -> Option<ShadowEntry> {
        self.stack.pop()
    }

    pub(crate) fn take(&self, n: usize, last_pc: u32) -> Backtrace {
        let mut stack = self
            .stack
            .iter()
            .rev()
            .take(n)
            .rev()
            .cloned()
            .collect::<ThinVec<_>>();

        if let Some(ShadowEntry::Bytecode { pc, .. }) = stack.last_mut() {
            // NOTE: pc points to the next opcode, so we offset by -1 to put it within range.
            *pc = last_pc.saturating_sub(1);
        }
        Backtrace { stack }
    }

    pub(crate) fn take_and_push(&self, n: usize, last_pc: u32, value: ShadowEntry) -> Backtrace {
        let mut stack = self
            .stack
            .iter()
            .rev()
            .take(n)
            .rev()
            .cloned()
            .chain(std::iter::once(value))
            .collect::<ThinVec<_>>();

        let last = stack.len() - 2;
        if let Some(ShadowEntry::Bytecode { pc, .. }) = stack.get_mut(last) {
            // NOTE: pc points to the next opcode, so we offset by -1 to put it within range.
            *pc = last_pc.saturating_sub(1);
        }
        Backtrace { stack }
    }

    pub(crate) fn caller_position(&self, n: usize) -> Backtrace {
        // NOTE: We push the function that is currently executing, so skip the last one.
        let stack = self
            .stack
            .iter()
            .rev()
            .skip(1)
            .take(n)
            .rev()
            .cloned()
            .collect::<ThinVec<_>>();

        Backtrace { stack }
    }

    #[cfg(feature = "native-backtrace")]
    pub(crate) fn patch_last_native(&mut self, new_source_info: NativeSourceInfo) {
        let Some(ShadowEntry::Native { source_info, .. }) = self.stack.last_mut() else {
            return;
        };
        *source_info = new_source_info;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restoring_an_invocation_preserves_the_callers_bytecode_position() {
        let mut stack = ShadowStack::default();
        stack.push_bytecode(0, SourceInfo::default());
        stack.push_bytecode(12, SourceInfo::default());
        let checkpoint = stack.checkpoint();
        stack.push_native(24, JsString::from("nested"), NativeSourceInfo::caller());
        stack.push_bytecode(36, SourceInfo::default());

        stack.restore(checkpoint);

        assert!(matches!(
            stack.pop(),
            Some(ShadowEntry::Bytecode { pc: 0, .. })
        ));
        assert!(matches!(
            stack.pop(),
            Some(ShadowEntry::Bytecode { pc: 11, .. })
        ));
        assert!(stack.pop().is_none());
    }

    #[test]
    fn restoring_an_invocation_preserves_the_callers_native_source_location() {
        let mut stack = ShadowStack::default();
        let caller = NativeSourceInfo::caller();
        stack.push_native(0, JsString::from("caller"), caller);
        let checkpoint = stack.checkpoint();
        let nested = NativeSourceInfo::caller();
        stack.push_native(0, JsString::from("nested"), nested);

        stack.restore(checkpoint);

        let Some(ShadowEntry::Native {
            function_name,
            source_info,
        }) = stack.pop()
        else {
            panic!("the original native caller must remain")
        };
        assert_eq!(function_name, Some(JsString::from("caller")));
        assert_eq!(source_info.as_location(), caller.as_location());
        assert!(stack.pop().is_none());
    }
}
