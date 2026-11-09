use crate::{
    JsResult,
    bytecompiler::control::{CompilerControl, CompilerWork, reserve, size_overflow},
    vm::opcode::RegisterOperand,
};

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
    pub(crate) fn alloc(&mut self, control: &mut CompilerControl<'_>) -> JsResult<Register> {
        for (i, register) in self.registers.iter_mut().enumerate() {
            control.admit(CompilerWork::RegisterProbe)?;
            if !register.flags.is_used() {
                assert!(!register.flags.is_persistent());
                let index = u32::try_from(i).map_err(|_| size_overflow())?;
                control.admit(CompilerWork::RegisterWrite)?;
                register.flags |= RegisterFlags::USED;
                return Ok(Register {
                    index,
                    flags: register.flags,
                });
            }
        }

        let flags = RegisterFlags::USED;

        let index = u32::try_from(self.registers.len()).map_err(|_| size_overflow())?;
        let count = self
            .registers
            .len()
            .checked_add(1)
            .ok_or_else(size_overflow)?;
        u32::try_from(count).map_err(|_| size_overflow())?;
        reserve(&mut self.registers, 1, control)?;
        control.admit(CompilerWork::RegisterWrite)?;
        self.registers.push(RegisterEntry { flags });

        Ok(Register { index, flags })
    }

    pub(crate) fn alloc_persistent(
        &mut self,
        control: &mut CompilerControl<'_>,
    ) -> JsResult<Register> {
        let mut reg = self.alloc(control)?;

        let index = reg.index();

        control.admit(CompilerWork::RegisterProbe)?;
        let register = &mut self.registers[index as usize];

        control.admit(CompilerWork::RegisterWrite)?;
        register.flags |= RegisterFlags::PERSISTENT;

        reg.flags = register.flags;
        Ok(reg)
    }

    #[track_caller]
    pub(crate) fn dealloc(
        &mut self,
        reg: Register,
        control: &mut CompilerControl<'_>,
    ) -> JsResult<()> {
        assert!(
            !reg.flags.is_persistent(),
            "Trying to deallocate a persistent register"
        );

        control.admit(CompilerWork::RegisterProbe)?;
        let register = &mut self.registers[reg.index as usize];

        assert!(
            register.flags.is_used(),
            "Cannot deallocate unused variable"
        );
        control.admit(CompilerWork::RegisterWrite)?;
        register.flags.set(RegisterFlags::USED, false);
        Ok(())
    }

    pub(crate) fn finish(self, control: &mut CompilerControl<'_>) -> JsResult<u32> {
        for register in &self.registers {
            control.admit(CompilerWork::RegisterProbe)?;
            assert!(
                !register.flags.is_used() || register.flags.is_persistent(),
                "finished compilation with an undeallocated temporary register"
            );
        }

        u32::try_from(self.registers.len()).map_err(|_| size_overflow())
    }
}

#[cfg(test)]
mod tests {
    use super::{RegisterAllocator, RegisterEntry, RegisterFlags};
    use crate::{
        JsError, JsValue,
        bytecompiler::control::{CompilerControl, CompilerDepth, CompilerWork},
    };
    use std::panic::catch_unwind;

    #[test]
    fn ordinary_refusal_with_live_temporaries_preserves_the_typed_error() {
        #[derive(Debug, PartialEq, Eq)]
        enum Refusal {
            RegisterBudget { next_register: u32 },
        }

        fn refuse() -> Result<(), Refusal> {
            let depth = CompilerDepth::new();
            let mut control = CompilerControl::new(None, &depth);
            let mut allocator = RegisterAllocator::default();
            let _first = allocator.alloc(&mut control).unwrap();
            let _second = allocator.alloc(&mut control).unwrap();
            Err(Refusal::RegisterBudget { next_register: 2 })
        }

        let result = catch_unwind(refuse)
            .expect("ordinary refusal must not panic while dropping live temporary registers");
        assert_eq!(result, Err(Refusal::RegisterBudget { next_register: 2 }));
    }

    #[test]
    fn successful_finish_rejects_an_outstanding_temporary() {
        let mut allocator = RegisterAllocator::default();
        let depth = CompilerDepth::new();
        let mut control = CompilerControl::new(None, &depth);
        // Keep the token live through finish and isolate its cleanup from the
        // allocator invariant being checked.
        let temporary = std::mem::ManuallyDrop::new(allocator.alloc(&mut control).unwrap());
        let finished = catch_unwind(move || {
            let depth = CompilerDepth::new();
            allocator.finish(&mut CompilerControl::new(None, &depth))
        });
        assert_eq!(temporary.index(), 0);
        assert!(
            finished.is_err(),
            "finish accepted an outstanding temporary"
        );
    }

    #[test]
    fn deallocation_reuses_temporaries_and_finish_retains_persistent_registers() {
        let mut allocator = RegisterAllocator::default();
        let depth = CompilerDepth::new();
        let mut control = CompilerControl::new(None, &depth);
        let persistent = allocator.alloc_persistent(&mut control).unwrap();
        let first = allocator.alloc(&mut control).unwrap();
        let second = allocator.alloc(&mut control).unwrap();
        assert_eq!(persistent.index(), 0);
        assert_eq!(first.index(), 1);
        assert_eq!(second.index(), 2);

        allocator.dealloc(first, &mut control).unwrap();
        let reused = allocator.alloc(&mut control).unwrap();
        assert_eq!(reused.index(), 1);
        allocator.dealloc(second, &mut control).unwrap();
        allocator.dealloc(reused, &mut control).unwrap();

        assert_eq!(allocator.finish(&mut control).unwrap(), 3);
    }

    #[test]
    fn wide_reuse_admits_each_probe_and_the_write_before_changing_flags() {
        const WIDTH: usize = 32;
        let marker = JsError::from_opaque(JsValue::new(211)).unwrap();
        // The last entry is free: 32 probes and one flag write are necessary.
        for limit in [WIDTH, WIDTH + 1] {
            let mut allocator = RegisterAllocator {
                registers: (0..WIDTH)
                    .map(|index| RegisterEntry {
                        flags: if index + 1 == WIDTH {
                            RegisterFlags::empty()
                        } else {
                            RegisterFlags::USED | RegisterFlags::PERSISTENT
                        },
                    })
                    .collect(),
            };
            let mut accepted = 0;
            let mut probes = 0;
            let mut admission = |work| {
                match work {
                    CompilerWork::RegisterProbe => probes += 1,
                    CompilerWork::RegisterWrite => {}
                    _ => panic!("reuse needs no storage growth"),
                }
                if accepted == limit {
                    return Err(marker.clone());
                }
                accepted += 1;
                Ok(())
            };
            let depth = CompilerDepth::new();
            let result = allocator.alloc(&mut CompilerControl::new(Some(&mut admission), &depth));
            assert_eq!(probes, WIDTH);
            assert_eq!(accepted, limit);
            if limit == WIDTH + 1 {
                assert_eq!(result.unwrap().index(), (WIDTH - 1) as u32);
                assert!(allocator.registers[WIDTH - 1].flags.is_used());
            } else {
                assert_eq!(result.unwrap_err(), marker);
                assert!(!allocator.registers[WIDTH - 1].flags.is_used());
            }
        }
    }

    #[test]
    fn a_probe_refusal_stops_before_later_entries_or_any_flag_write() {
        let mut allocator = RegisterAllocator {
            registers: (0..8)
                .map(|_| RegisterEntry {
                    flags: RegisterFlags::USED | RegisterFlags::PERSISTENT,
                })
                .collect(),
        };
        let marker = JsError::from_opaque(JsValue::new(212)).unwrap();
        let mut probes = 0;
        let mut admission = |work| {
            assert!(matches!(work, CompilerWork::RegisterProbe));
            probes += 1;
            if probes == 3 {
                Err(marker.clone())
            } else {
                Ok(())
            }
        };
        let depth = CompilerDepth::new();
        let error = allocator
            .alloc(&mut CompilerControl::new(Some(&mut admission), &depth))
            .unwrap_err();
        assert_eq!(error, marker);
        assert_eq!(probes, 3);
        assert_eq!(allocator.registers.len(), 8);
    }

    #[test]
    fn persistent_flag_refusal_returns_the_original_error_and_leaves_cleanup_infallible() {
        let marker = JsError::from_opaque(JsValue::new(215)).unwrap();
        for limit in [2, 3] {
            let mut allocator = RegisterAllocator {
                registers: Vec::with_capacity(1),
            };
            let mut accepted = 0;
            let mut admission = |work| {
                assert!(matches!(
                    work,
                    CompilerWork::RegisterProbe | CompilerWork::RegisterWrite
                ));
                if accepted == limit {
                    return Err(marker.clone());
                }
                accepted += 1;
                Ok(())
            };
            let depth = CompilerDepth::new();
            let result =
                allocator.alloc_persistent(&mut CompilerControl::new(Some(&mut admission), &depth));
            assert_eq!(accepted, limit);
            assert_eq!(allocator.registers.len(), 1);
            assert!(allocator.registers[0].flags.is_used());
            if limit == 3 {
                assert!(result.unwrap().flags.is_persistent());
                assert_eq!(
                    allocator
                        .finish(&mut CompilerControl::new(None, &depth))
                        .unwrap(),
                    1
                );
            } else {
                assert_eq!(result.unwrap_err(), marker);
                assert!(!allocator.registers[0].flags.is_persistent());
                drop(allocator);
            }
        }
    }

    #[test]
    fn new_register_refusal_before_growth_does_not_append_an_entry() {
        let mut allocator = RegisterAllocator::default();
        let marker = JsError::from_opaque(JsValue::new(216)).unwrap();
        let mut requests = Vec::new();
        let mut admission = |work| {
            requests.push(work);
            Err(marker.clone())
        };
        let depth = CompilerDepth::new();
        assert_eq!(
            allocator
                .alloc(&mut CompilerControl::new(Some(&mut admission), &depth))
                .unwrap_err(),
            marker
        );
        assert!(allocator.registers.is_empty());
        assert!(
            matches!(requests.as_slice(), [CompilerWork::Capacity { bytes }] if *bytes == size_of::<RegisterEntry>())
        );
    }

    #[test]
    fn refused_deallocation_and_finish_can_discard_live_tokens_without_callbacks() {
        let marker = JsError::from_opaque(JsValue::new(213)).unwrap();
        let depth = CompilerDepth::new();
        let mut allocator = RegisterAllocator::default();
        let register = allocator
            .alloc(&mut CompilerControl::new(None, &depth))
            .unwrap();
        let mut requests = 0;
        let mut refusal = |_| {
            requests += 1;
            Err(marker.clone())
        };
        let result = allocator.dealloc(
            register,
            &mut CompilerControl::new(Some(&mut refusal), &depth),
        );
        assert_eq!(result.unwrap_err(), marker);
        assert!(allocator.registers[0].flags.is_used());
        drop(allocator);
        assert_eq!(requests, 1);

        let mut allocator = RegisterAllocator::default();
        let _temporary = allocator
            .alloc(&mut CompilerControl::new(None, &depth))
            .unwrap();
        let mut requests = 0;
        let mut refusal = |_| {
            requests += 1;
            Err(marker.clone())
        };
        assert_eq!(
            allocator
                .finish(&mut CompilerControl::new(Some(&mut refusal), &depth))
                .unwrap_err(),
            marker
        );
        assert_eq!(requests, 1);
    }

    #[test]
    fn finish_visits_each_persistent_entry_at_exact_and_one_short_limits() {
        for limit in [3, 4] {
            let allocator = RegisterAllocator {
                registers: (0..4)
                    .map(|_| RegisterEntry {
                        flags: RegisterFlags::USED | RegisterFlags::PERSISTENT,
                    })
                    .collect(),
            };
            let marker = JsError::from_opaque(JsValue::new(214)).unwrap();
            let mut accepted = 0;
            let mut admission = |work| {
                assert!(matches!(work, CompilerWork::RegisterProbe));
                if accepted == limit {
                    return Err(marker.clone());
                }
                accepted += 1;
                Ok(())
            };
            let depth = CompilerDepth::new();
            let result = allocator.finish(&mut CompilerControl::new(Some(&mut admission), &depth));
            assert_eq!(accepted, limit);
            if limit == 4 {
                assert_eq!(result.unwrap(), 4);
            } else {
                assert_eq!(result.unwrap_err(), marker);
            }
        }
    }
}
