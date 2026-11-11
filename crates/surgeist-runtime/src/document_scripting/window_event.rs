use std::{cell::RefCell, mem::size_of, rc::Rc};

use surgeist_bindings_js::{
    Arguments, Callback, Completion, Error, FunctionMetadata, Primitive, PropertyDescriptor,
    PropertyKey, Root, Scope,
};
use surgeist_dom::DomHostedRuntimeRetention;

const EVENT: [u8; 5] = *b"event";
const ILLEGAL_RECEIVER: [u8; 19] = *b"Not a Window object";

pub(super) struct WindowEvent {
    // Replacing the guest property leaves the actual per-callback slot intact.
    current: RefCell<Option<Root>>,
    _backing: DomHostedRuntimeRetention,
}

impl WindowEvent {
    pub(super) const fn storage_bytes() -> usize {
        // One shared state and the two native callback captures.
        size_of::<Self>() + 2 * size_of::<Rc<Self>>()
    }

    pub(super) fn new(backing: DomHostedRuntimeRetention) -> Rc<Self> {
        Rc::new(Self {
            current: RefCell::new(None),
            _backing: backing,
        })
    }

    pub(super) fn install(self: &Rc<Self>, scope: &mut Scope<'_>) -> Result<(), Error> {
        let getter = self.clone();
        let get = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| getter.get(scope, arguments)),
            FunctionMetadata {
                name: &b"get event".map(u16::from),
                length: 0,
            },
        )?;
        let setter = self.clone();
        let set = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| setter.set(scope, arguments)),
            FunctionMetadata {
                name: &b"set event".map(u16::from),
                length: 1,
            },
        )?;
        let global = scope.global()?;
        scope.define_new_property(
            &global,
            PropertyKey::String(&EVENT.map(u16::from)),
            PropertyDescriptor::Accessor {
                get: Some(&get),
                set: Some(&set),
                enumerable: true,
                configurable: true,
            },
        )
    }

    fn get<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let global = scope.global()?;
        if !scope.strict_equals(arguments.receiver(), &global)? {
            return Ok(Completion::Throw(
                scope.type_error(&ILLEGAL_RECEIVER.map(u16::from))?,
            ));
        }
        let current = self.current.borrow().clone();
        match current {
            Some(event) => Ok(Completion::Return(scope.import(&event)?)),
            None => Ok(Completion::Undefined),
        }
    }

    fn set<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let global = scope.global()?;
        if !scope.strict_equals(arguments.receiver(), &global)? {
            return Ok(Completion::Throw(
                scope.type_error(&ILLEGAL_RECEIVER.map(u16::from))?,
            ));
        }
        let undefined;
        let value = match arguments.get(0) {
            Some(value) => value,
            None => {
                undefined = scope.primitive(Primitive::Undefined)?;
                &undefined
            }
        };
        match scope.define_property(
            &global,
            PropertyKey::String(&EVENT.map(u16::from)),
            PropertyDescriptor::Data {
                value,
                writable: true,
                enumerable: true,
                configurable: true,
            },
        ) {
            Ok(()) => Ok(Completion::Undefined),
            Err(Error::PropertyRejected) => Ok(Completion::Throw(
                scope.type_error(&b"Cannot replace Window.event".map(u16::from))?,
            )),
            Err(error) => Err(error),
        }
    }

    pub(super) fn enter(&self, event: Root) -> CurrentEvent<'_> {
        CurrentEvent {
            slot: &self.current,
            previous: self.current.replace(Some(event)),
        }
    }

    #[cfg(test)]
    pub(super) fn is_idle(&self) -> bool {
        self.current.borrow().is_none()
    }
}

pub(super) struct CurrentEvent<'a> {
    slot: &'a RefCell<Option<Root>>,
    previous: Option<Root>,
}

impl Drop for CurrentEvent<'_> {
    fn drop(&mut self) {
        self.slot.replace(self.previous.take());
    }
}
