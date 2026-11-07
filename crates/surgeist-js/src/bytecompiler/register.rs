use crate::vm::opcode::RegisterOperand;

bitflags::bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    struct RegisterFlags: u8 {
        /// Whether the register is still in use (not deallocated).
        const USED       = 0b0000_0001;

        const PERSISTENT = 0b0000_0010;
    }
}

impl RegisterFlags {
    fn is_used(self) -> bool {
        self.contains(Self::USED)
    }
    fn is_persistent(self) -> bool {
        self.contains(Self::PERSISTENT)
    }
}

/// An entry in the [`RegisterAllocator`].
#[derive(Debug, Default)]
pub(crate) struct RegisterEntry {
    flags: RegisterFlags,
}

/// Represent a VM register.
///
/// This token is passed by reference or moved. Dropping it leaves the allocator's
/// entry in use, so interrupted compilation can discard its private allocator
/// without cleanup panics. Successful compilation must deallocate temporary
/// registers before [`RegisterAllocator::finish()`]; persistent registers remain
/// allocated until that allocator is discarded.
///
/// A [`Register`] is index into the register allocator,
/// as well as an index into the per-frame register file on each [`CallFrame`](crate::vm::CallFrame).
#[derive(Debug)]
pub(crate) struct Register {
    index: u32,
    flags: RegisterFlags,
}

impl Register {
    /// The index of the [`Register`].
    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    /// The index of the [`Register`] as a [`RegisterOperand`].
    pub(crate) fn variable(&self) -> RegisterOperand {
        self.index.into()
    }

    pub(crate) fn persistent(index: u32) -> Self {
        Self {
            index,
            flags: RegisterFlags::PERSISTENT,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct RegisterAllocator {
    registers: Vec<RegisterEntry>,
}

impl RegisterAllocator {
    pub(crate) fn alloc(&mut self) -> Register {
        if let Some((i, register)) = self
            .registers
            .iter_mut()
            .enumerate()
            .find(|(_, reg)| !reg.flags.is_used())
        {
            assert!(!register.flags.is_persistent());

            register.flags |= RegisterFlags::USED;
            return Register {
                index: i as u32,
                flags: register.flags,
            };
        }

        let flags = RegisterFlags::USED;

        let index = self.registers.len() as u32;
        self.registers.push(RegisterEntry { flags });

        Register { index, flags }
    }

    pub(crate) fn alloc_persistent(&mut self) -> Register {
        let mut reg = self.alloc();

        let index = reg.index();

        let register = &mut self.registers[index as usize];

        register.flags |= RegisterFlags::PERSISTENT;

        reg.flags = register.flags;
        reg
    }

    #[track_caller]
    pub(crate) fn dealloc(&mut self, reg: Register) {
        assert!(
            !reg.flags.is_persistent(),
            "Trying to deallocate a persistent register"
        );

        let register = &mut self.registers[reg.index as usize];

        assert!(
            register.flags.is_used(),
            "Cannot deallocate unused variable"
        );
        register.flags.set(RegisterFlags::USED, false);
    }

    pub(crate) fn finish(self) -> u32 {
        for register in &self.registers {
            assert!(
                !register.flags.is_used() || register.flags.is_persistent(),
                "finished compilation with an undeallocated temporary register"
            );
        }

        self.registers.len() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::RegisterAllocator;
    use std::panic::catch_unwind;

    #[test]
    fn ordinary_refusal_with_live_temporaries_preserves_the_typed_error() {
        #[derive(Debug, PartialEq, Eq)]
        enum Refusal {
            RegisterBudget { next_register: u32 },
        }

        fn refuse() -> Result<(), Refusal> {
            let mut allocator = RegisterAllocator::default();
            let _first = allocator.alloc();
            let _second = allocator.alloc();
            Err(Refusal::RegisterBudget { next_register: 2 })
        }

        let result = catch_unwind(refuse)
            .expect("ordinary refusal must not panic while dropping live temporary registers");
        assert_eq!(result, Err(Refusal::RegisterBudget { next_register: 2 }));
    }

    #[test]
    fn successful_finish_rejects_an_outstanding_temporary() {
        let mut allocator = RegisterAllocator::default();
        // Keep the token live through finish and isolate its cleanup from the
        // allocator invariant being checked.
        let temporary = std::mem::ManuallyDrop::new(allocator.alloc());
        let finished = catch_unwind(move || allocator.finish());
        assert_eq!(temporary.index(), 0);
        assert!(
            finished.is_err(),
            "finish accepted an outstanding temporary"
        );
    }

    #[test]
    fn deallocation_reuses_temporaries_and_finish_retains_persistent_registers() {
        let mut allocator = RegisterAllocator::default();
        let persistent = allocator.alloc_persistent();
        let first = allocator.alloc();
        let second = allocator.alloc();
        assert_eq!(persistent.index(), 0);
        assert_eq!(first.index(), 1);
        assert_eq!(second.index(), 2);

        allocator.dealloc(first);
        let reused = allocator.alloc();
        assert_eq!(reused.index(), 1);
        allocator.dealloc(second);
        allocator.dealloc(reused);

        assert_eq!(allocator.finish(), 3);
    }
}
