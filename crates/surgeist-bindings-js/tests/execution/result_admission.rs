use super::units;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use surgeist_bindings_js::preparation::{Admission, BufferKind, Request, Retention, Work};
use surgeist_bindings_js::{Error, Primitive, Process};

#[derive(Debug, PartialEq, Eq)]
enum BackendPreparation {
    ResultRoot,
    OutboundLocals(usize),
    OutboundLocal,
}

#[derive(Default)]
struct Accounting {
    reached: RefCell<Vec<BackendPreparation>>,
    live: Cell<usize>,
}

struct Backing {
    accounting: Rc<Accounting>,
    bytes: usize,
}

impl Retention for Backing {
    fn release(self: Box<Self>) -> Result<(), Error> {
        let live = self
            .accounting
            .live
            .get()
            .checked_sub(self.bytes)
            .ok_or(Error::RegistryConflict)?;
        self.accounting.live.set(live);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    Array,
    Call,
    ConstructNonconstructor,
}

fn check(operation: Operation, refuse_root: bool) -> Result<(), Error> {
    let mut process = Process::bootstrap()?;
    let realm = process.execution().create_realm()?;
    let (function, receiver, argument) = process.execution().in_realm(&realm, |scope| {
        let function = scope.evaluate(&units("value => value"))?;
        let receiver = scope.primitive(Primitive::Undefined)?;
        let argument = scope.primitive(Primitive::Boolean(true))?;
        Ok((
            scope.retain(&function),
            scope.retain(&receiver),
            scope.retain(&argument),
        ))
    })?;

    let accounting = Rc::new(Accounting::default());
    let admitted = accounting.clone();
    let charged = accounting.clone();
    // Scope materializes OutboundRoots before calling the backend. Observe the
    // backend result root and OutboundLocals, including each actual local copy.
    let controller = Admission::new(
        move |request| {
            let reached = match request {
                Request::RootRecord { .. } => Some(BackendPreparation::ResultRoot),
                Request::Buffer {
                    kind: BufferKind::OutboundLocals,
                    elements,
                    ..
                } => Some(BackendPreparation::OutboundLocals(elements)),
                _ => None,
            };
            if let Some(reached) = reached {
                admitted.reached.borrow_mut().push(reached);
            }
            if refuse_root && matches!(request, Request::RootRecord { .. }) {
                return Err(Error::Allocation);
            }
            let bytes = request.bytes();
            admitted.live.set(admitted.live.get() + bytes);
            Ok(Backing {
                accounting: admitted.clone(),
                bytes,
            })
        },
        move |work| {
            if work == Work::BufferElement(BufferKind::OutboundLocals) {
                charged
                    .reached
                    .borrow_mut()
                    .push(BackendPreparation::OutboundLocal);
            }
            Ok(())
        },
    )
    .install(&mut process, ())?;

    let entered = Cell::new(false);
    let result = process.execution().in_realm(&realm, |scope| {
        let function = scope.import(&function)?;
        let receiver = scope.import(&receiver)?;
        let argument = scope.import(&argument)?;
        match operation {
            Operation::Array => scope.array(&[argument]).map(drop),
            Operation::Call => scope
                .call_with_entry(&function, &receiver, &[argument], || {
                    assert!(!entered.replace(true));
                    Ok(())
                })
                .map(drop),
            Operation::ConstructNonconstructor => {
                scope.construct_without_arguments(&receiver).map(drop)
            }
        }
    });

    let expected = match (operation, refuse_root) {
        (_, true) => {
            assert!(matches!(result, Err(Error::Allocation)), "{operation:?}");
            &[BackendPreparation::ResultRoot][..]
        }
        (Operation::ConstructNonconstructor, false) => {
            assert!(matches!(result, Err(Error::ExpectedConstructor)));
            &[BackendPreparation::ResultRoot][..]
        }
        (Operation::Array | Operation::Call, false) => {
            result?;
            &[
                BackendPreparation::ResultRoot,
                BackendPreparation::OutboundLocals(1),
                BackendPreparation::OutboundLocal,
            ][..]
        }
    };
    assert_eq!(
        accounting.reached.borrow().as_slice(),
        expected,
        "{operation:?}"
    );
    assert_eq!(
        entered.get(),
        matches!(operation, Operation::Call) && !refuse_root
    );
    assert_eq!(accounting.live.get(), 0);
    assert!(controller.observation().cleanup_error().is_none());
    Ok(())
}

pub(super) fn results_admit_before_backend_preparation() -> Result<(), Error> {
    for operation in [
        Operation::Array,
        Operation::Call,
        Operation::ConstructNonconstructor,
    ] {
        check(operation, false)?;
        check(operation, true)?;
    }
    Ok(())
}
