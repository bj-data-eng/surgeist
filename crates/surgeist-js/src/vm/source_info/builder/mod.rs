use std::{cmp::Ordering, ops::Range};

use surgeist_js_ast::Position;

use crate::{
    JsResult,
    bytecompiler::control::{CompilerControl, CompilerWork, payload_bytes, reserve},
    vm::source_info::Entry,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EntryRange {
    start: u32,
    end: u32,
    position: Option<Position>,
}

impl EntryRange {
    fn range(&self) -> Range<u32> {
        self.start..self.end
    }

    fn is_empty(&self) -> bool {
        self.range().is_empty()
    }
}

/// A live scope and the parent state to restore if this scope is abandoned.
#[derive(Debug)]
struct SourceScope {
    index: usize,
    entries_start: usize,
    parent: Option<(usize, EntryRange)>,
}

enum ParentContinuation {
    Swap {
        parent: usize,
    },
    Split {
        parent: usize,
        index: usize,
        position: Option<Position>,
    },
}

#[derive(Debug, Default)]
pub(crate) struct SourceMapBuilder {
    entries: Vec<EntryRange>,
    stack: Vec<SourceScope>,
}

fn reserve_push<T>(values: &mut Vec<T>, control: &mut CompilerControl<'_>) -> JsResult<()> {
    // Request geometric capacity instead of relocating the complete metadata
    // payload for each new range. The shared owner admits the actual request.
    let additional = if values.len() == values.capacity() {
        values.len().max(1)
    } else {
        1
    };
    reserve(values, additional, control)
}

impl SourceMapBuilder {
    pub(crate) fn build(
        self,
        final_pc: u32,
        control: &mut CompilerControl<'_>,
    ) -> JsResult<Box<[Entry]>> {
        control.admit(CompilerWork::MetadataVisit)?;
        assert!(self.stack.is_empty(), "forgot to pop source scope");
        control.admit(CompilerWork::MetadataVisit)?;
        let end_entry = if let Some(entry) = self.entries.last() {
            control.admit(CompilerWork::Read {
                bytes: payload_bytes::<u32>(1)?,
            })?;
            control.admit(CompilerWork::MetadataWrite)?;
            EntryRange {
                start: entry.end,
                end: final_pc,
                position: None,
            }
        } else {
            control.admit(CompilerWork::MetadataWrite)?;
            EntryRange::default()
        };
        let mut output = Vec::<Entry>::new();
        for entry in self.entries.iter().chain(std::iter::once(&end_entry)) {
            control.admit(CompilerWork::MetadataVisit)?;
            control.admit(CompilerWork::Read {
                bytes: payload_bytes::<EntryRange>(1)?,
            })?;
            if entry.is_empty() {
                continue;
            }
            control.admit(CompilerWork::MetadataVisit)?;
            if let Some(previous) = output.last() {
                control.admit(CompilerWork::Read {
                    bytes: payload_bytes::<Option<Position>>(1)?,
                })?;
                if previous.position == entry.position {
                    continue;
                }
            }
            reserve_push(&mut output, control)?;
            control.admit(CompilerWork::Copy {
                bytes: payload_bytes::<Entry>(1)?,
            })?;
            control.admit(CompilerWork::MetadataWrite)?;
            output.push(Entry {
                pc: entry.start,
                position: entry.position,
            });
        }
        if output.capacity() != output.len() {
            let bytes = payload_bytes::<Entry>(output.len())?;
            control.admit(CompilerWork::Capacity { bytes })?;
            control.admit(CompilerWork::Copy { bytes })?;
        }
        control.admit(CompilerWork::MetadataWrite)?;
        Ok(output.into_boxed_slice())
    }

    pub(crate) fn push_source_position(
        &mut self,
        start_pc: u32,
        position: Option<Position>,
        control: &mut CompilerControl<'_>,
    ) -> JsResult<()> {
        control.admit(CompilerWork::MetadataVisit)?;
        let parent = if let Some(scope) = self.stack.last() {
            control.admit(CompilerWork::Read {
                bytes: payload_bytes::<usize>(1)?,
            })?;
            let index = scope.index;
            control.admit(CompilerWork::MetadataVisit)?;
            control.admit(CompilerWork::Read {
                bytes: payload_bytes::<EntryRange>(1)?,
            })?;
            control.admit(CompilerWork::Copy {
                bytes: payload_bytes::<EntryRange>(1)?,
            })?;
            Some((index, self.entries[index]))
        } else {
            None
        };
        reserve_push(&mut self.entries, control)?;
        reserve_push(&mut self.stack, control)?;
        control.admit(CompilerWork::MetadataWrite)?;
        control.admit(CompilerWork::MetadataWrite)?;
        let index = self.entries.len();
        self.entries.push(EntryRange {
            start: start_pc,
            end: u32::MAX,
            position,
        });
        self.stack.push(SourceScope {
            index,
            entries_start: index,
            parent,
        });
        Ok(())
    }

    /// Abandons the current scope without completing ranges or allocating metadata.
    /// Nested completed ranges belonging to this scope are discarded as well.
    pub(crate) fn abort_source_position(&mut self) {
        let Some(scope) = self.stack.pop() else {
            panic!("aborted more than pushed");
        };
        self.entries.truncate(scope.entries_start);
        if let Some((index, entry)) = scope.parent {
            self.entries[index] = entry;
            if let Some(parent) = self.stack.last_mut() {
                parent.index = index;
            }
        }
    }

    /// Completes the child range and resumes its parent's source position.
    /// Equal starts swap the ranges; a later child splits its parent and appends
    /// a continuation. All fallible work precedes changes to the live scopes.
    pub(crate) fn pop_source_position(
        &mut self,
        current_start_pc: u32,
        control: &mut CompilerControl<'_>,
    ) -> JsResult<()> {
        control.admit(CompilerWork::MetadataVisit)?;
        let Some(scope) = self.stack.last() else {
            panic!("popped more than pushed");
        };
        control.admit(CompilerWork::Read {
            bytes: payload_bytes::<usize>(1)?,
        })?;
        let index = scope.index;
        control.admit(CompilerWork::MetadataVisit)?;
        control.admit(CompilerWork::Read {
            bytes: payload_bytes::<u32>(1)?,
        })?;
        let start = self.entries[index].start;
        let mut continuation = None;
        if start < current_start_pc {
            control.admit(CompilerWork::MetadataVisit)?;
            if let Some(parent) = self
                .stack
                .len()
                .checked_sub(2)
                .and_then(|index| self.stack.get(index))
            {
                control.admit(CompilerWork::Read {
                    bytes: payload_bytes::<usize>(1)?,
                })?;
                let parent = parent.index;
                control.admit(CompilerWork::MetadataVisit)?;
                control.admit(CompilerWork::Read {
                    bytes: payload_bytes::<EntryRange>(1)?,
                })?;
                match self.entries[parent].start.cmp(&start) {
                    Ordering::Equal => {
                        control.admit(CompilerWork::Copy {
                            bytes: payload_bytes::<EntryRange>(2)?,
                        })?;
                        continuation = Some(ParentContinuation::Swap { parent });
                    }
                    Ordering::Less => {
                        assert_eq!(
                            self.entries[parent].end,
                            u32::MAX,
                            "parent end position should not be set"
                        );
                        control.admit(CompilerWork::Copy {
                            bytes: payload_bytes::<Option<Position>>(1)?,
                        })?;
                        let position = self.entries[parent].position;
                        reserve_push(&mut self.entries, control)?;
                        continuation = Some(ParentContinuation::Split {
                            parent,
                            index: self.entries.len(),
                            position,
                        });
                    }
                    Ordering::Greater => {
                        unreachable!("Parent source scope cannot be greater than child scope")
                    }
                }
            }
        }
        // Admission, including any split storage, finishes while the child is
        // still the live scope. A guard can abort it after any refused request.
        let writes = match &continuation {
            // End the child range and remove its scope.
            None => 2,
            // Also replace both ranges, resume the parent, and update its scope.
            Some(ParentContinuation::Swap { .. }) => 6,
            // Also shorten the parent, append its continuation, and update its scope.
            Some(ParentContinuation::Split { .. }) => 5,
        };
        for _ in 0..writes {
            control.admit(CompilerWork::MetadataWrite)?;
        }
        self.stack
            .pop()
            .expect("the inspected source scope remains live");
        self.entries[index].end = current_start_pc;
        if let Some(continuation) = continuation {
            let new_parent_index = match continuation {
                ParentContinuation::Split {
                    parent,
                    index: new_index,
                    position,
                } => {
                    self.entries[parent].end = start;
                    self.entries.push(EntryRange {
                        start: current_start_pc,
                        end: u32::MAX,
                        position,
                    });
                    new_index
                }
                ParentContinuation::Swap { parent } => {
                    self.entries.swap(parent, index);
                    self.entries[index].start = current_start_pc;
                    index
                }
            };
            self.stack
                .last_mut()
                .expect("the inspected parent remains live")
                .index = new_parent_index;
        }
        Ok(())
    }
}
