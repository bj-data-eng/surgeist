use super::{DropReceipt, evaluate, units};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use surgeist_bindings_js::conversion::{Resource, Work};
use surgeist_bindings_js::{
    Callback, Completion, EngineKind, Error, Execution, ObjectPrototype, Primitive, Process,
    PropertyKey, Realm, RetainedCallbackObject, Root, Scope,
};
use surgeist_ledger::{Change, Limits, Operation};

type Account = Rc<RefCell<Operation<Work, Resource>>>;

struct Backing {
    account: Account,
    bytes: u64,
}

impl Backing {
    fn admit(account: &Account, bytes: usize) -> Result<Self, Error> {
        let bytes = u64::try_from(bytes).unwrap();
        let mut ledger = account.borrow_mut();
        let root = ledger.root();
        ledger
            .apply(root, Change::Acquire(Resource::BufferBytes, bytes))
            .map_err(Error::ConversionLedger)?;
        Ok(Self {
            account: account.clone(),
            bytes,
        })
    }
}

impl Drop for Backing {
    fn drop(&mut self) {
        let mut ledger = self.account.borrow_mut();
        let root = ledger.root();
        ledger
            .apply(root, Change::Release(Resource::BufferBytes, self.bytes))
            .unwrap();
    }
}

fn account(cap: u64) -> Account {
    Rc::new(RefCell::new(
        Operation::new(
            "guest callback root",
            Limits::from_fn(|_| u64::MAX, |_| cap),
            1,
        )
        .unwrap(),
    ))
}

fn live(account: &Account) -> u64 {
    let ledger = account.borrow();
    ledger
        .usage(ledger.root())
        .unwrap()
        .live(Resource::BufferBytes)
}

fn retain_in(execution: &mut Execution<'_>, realm: &Realm, original: &Root) -> Result<Root, Error> {
    execution.in_realm(realm, |scope| {
        let value = scope.import(original)?;
        let object = scope.object(ObjectPrototype::Default)?;
        let key = PropertyKey::String(&[b'x'.into()]);
        scope.set(&object, key, &value)?;
        // Re-read to obtain this Realm's own distinct retaining root.
        let read = scope.get(&object, key)?;
        Ok(scope.retain(&read))
    })
}

fn price(scope: &mut Scope<'_>, root: &Root) -> Result<u64, Error> {
    let value = scope.import(root)?;
    let observed = scope.callback_object(&value)?;
    let mut price = None;
    let result = scope.retain_callback_object::<Backing, Error>(observed, |bytes| {
        price = Some(u64::try_from(bytes).unwrap());
        Err(Error::Allocation)
    });
    assert!(matches!(result, Err(Error::Allocation)));
    Ok(price.unwrap())
}

pub(super) fn native_projection_uses_the_retained_prototype_realm(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let origin = execution.create_realm()?;
    let receiver = execution.create_realm()?;
    let prototype = Rc::new(execution.in_realm(&origin, |scope| {
        let value = scope.object(ObjectPrototype::Default)?;
        let observed = scope.callback_object(&value)?;
        scope.retain_callback_object(observed, |_| Ok::<(), Error>(()))
    })?);
    let origin_id = prototype.associated_realm().identity();
    let receiving_id =
        execution.in_realm(&receiver, |scope| Ok(scope.realm_authority()?.identity()))?;
    assert_ne!(origin_id, receiving_id);

    let result = Rc::new(RefCell::new(None));
    let target = result.clone();
    let selected = prototype.clone();
    execution.in_realm(&receiver, |scope| {
        let project = scope.function(&Callback::new(move |scope, _| {
            assert_eq!(scope.realm_authority()?.identity(), receiving_id);
            let discovery = account(u64::MAX);
            let mut prices = [0, 0];
            let refused = scope.project_retained_native_object_in_prototype_realm(
                &selected,
                42_u32,
                |payload, holder| {
                    prices = [payload, holder].map(|bytes| u64::try_from(bytes).unwrap());
                    let _payload = Backing::admit(&discovery, payload)?;
                    Err::<(Backing, Backing), _>(Error::Allocation)
                },
            );
            assert!(matches!(refused, Err(Error::Allocation)));
            assert_eq!(live(&discovery), 0);
            let exact = prices.iter().sum::<u64>();
            let short = account(exact - 1);
            assert!(matches!(
                scope.project_retained_native_object_in_prototype_realm(
                    &selected,
                    42_u32,
                    |payload, holder| Ok((
                        Backing::admit(&short, payload)?,
                        Backing::admit(&short, holder)?
                    ))
                ),
                Err(Error::ConversionLedger(_))
            ));
            assert_eq!(live(&short), 0);
            let charged = account(exact);
            let holder = scope.project_retained_native_object_in_prototype_realm(
                &selected,
                42_u32,
                |payload, holder| {
                    Ok::<_, Error>((
                        Backing::admit(&charged, payload)?,
                        Backing::admit(&charged, holder)?,
                    ))
                },
            )?;
            assert_eq!(holder.associated_realm().identity(), origin_id);
            assert_eq!(scope.realm_authority()?.identity(), receiving_id);
            assert_eq!(live(&charged), exact);
            target.replace(Some((holder, charged, prices[0])));
            Ok(Completion::Undefined)
        }))?;
        let undefined = scope.primitive(Primitive::Undefined)?;
        scope.call(&project, &undefined, &[])?;
        Ok(())
    })?;

    let (holder, charged, payload_bytes) = result.take().unwrap();
    let opened_in_receiver = execution.in_realm(&receiver, |scope| {
        let value = scope.open_callback_object(&holder)?;
        assert_eq!(
            scope.callback_object(&value)?.associated_realm().identity(),
            origin_id
        );
        assert_eq!(*scope.native_object::<u32>(&value)?.unwrap().payload(), 42);
        let prototype = scope.open_callback_object(&prototype)?;
        let check = scope.evaluate(&units(
            "(object, proto) => Object.getPrototypeOf(object) === proto",
        ))?;
        let undefined = scope.primitive(Primitive::Undefined)?;
        let retained = scope.retain(&value);
        let result = scope.call(&check, &undefined, &[value, prototype])?;
        let yes = scope.primitive(Primitive::Boolean(true))?;
        assert!(scope.strict_equals(&result, &yes)?);
        Ok(retained)
    })?;
    execution.in_realm(&origin, |scope| {
        let value = scope.open_callback_object(&holder)?;
        let receiving = scope.import(&opened_in_receiver)?;
        assert!(scope.strict_equals(&value, &receiving)?);
        Ok(())
    })?;
    drop((origin, opened_in_receiver));
    execution.in_realm(&receiver, |scope| {
        let still_admitted =
            scope.project_retained_native_object_in_prototype_realm(&prototype, (), |_, _| {
                Ok::<_, Error>(((), ()))
            })?;
        let projected = scope.open_callback_object(&still_admitted)?;
        assert_eq!(
            scope
                .callback_object(&projected)?
                .associated_realm()
                .identity(),
            origin_id
        );
        scope.release_callback_object(still_admitted)?;
        scope.release_callback_object(holder)?;
        assert_eq!(live(&charged), payload_bytes);
        Ok(())
    })?;
    evaluate(&mut execution, &receiver, "gc()")?;
    execution.memory_pressure();
    assert_eq!(live(&charged), 0);
    Ok(())
}

pub(super) fn native_projection_refuses_foreign_prototype_before_admission() -> Result<(), Error> {
    let prototype = {
        let mut owner = Process::bootstrap(EngineKind::V8)?;
        let mut execution = owner.execution();
        let realm = execution.create_realm()?;
        execution.in_realm(&realm, |scope| {
            let value = scope.object(ObjectPrototype::Default)?;
            let observed = scope.callback_object(&value)?;
            scope.retain_callback_object(observed, |_| Ok::<(), Error>(()))
        })?
    };
    let mut owner = Process::bootstrap(EngineKind::V8)?;
    let mut execution = owner.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let mut attempts = 0;
        let result =
            scope.project_retained_native_object_in_prototype_realm(&prototype, (), |_, _| {
                attempts += 1;
                Ok::<_, Error>(((), ()))
            });
        assert!(matches!(result, Err(Error::ForeignOwner)));
        assert_eq!(attempts, 0);
        Ok(())
    })
}

pub(super) fn associated_realm_and_exact_custody(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let a = execution.create_realm()?;
    let b = execution.create_realm()?;
    let a_id = execution.in_realm(&a, |scope| Ok(scope.realm_authority()?.identity()))?;
    let b_id = execution.in_realm(&b, |scope| Ok(scope.realm_authority()?.identity()))?;
    assert_ne!(a_id, b_id);
    let drops = Rc::new(Cell::new(0));
    let receipt = DropReceipt(drops.clone());
    let function = execution.in_realm(&a, |scope| {
        let marker = scope.function(&Callback::new(move |scope, _| {
            let _receipt = &receipt;
            assert_eq!(scope.realm_authority()?.identity(), a_id);
            Ok(Completion::Return(
                scope.primitive(Primitive::Number(41.0))?,
            ))
        }))?;
        let closure = scope.evaluate(&units("(marker => function () { return marker() })"))?;
        let undefined = scope.primitive(Primitive::Undefined)?;
        let function = scope.call(&closure, &undefined, &[marker])?;
        Ok(scope.retain(&function))
    })?;
    let object = execution.in_realm(&b, |scope| {
        let object = scope.evaluate(&units("Object.create(null)"))?;
        Ok(scope.retain(&object))
    })?;
    let function_in_b = retain_in(&mut execution, &b, &function)?;
    let object_in_a = retain_in(&mut execution, &a, &object)?;
    execution.in_realm(&a, |scope| {
        let value = scope.import(&object_in_a)?;
        assert_eq!(
            scope.callback_object(&value)?.associated_realm().identity(),
            b_id
        );
        let primitive = scope.primitive(Primitive::Null)?;
        assert!(matches!(
            scope.callback_object(&primitive),
            Err(Error::ExpectedObject)
        ));
        Ok(())
    })?;
    let exact = execution.in_realm(&b, |scope| price(scope, &function_in_b))?;
    assert!(exact > u64::try_from(size_of::<Backing>()).unwrap());
    let short = account(exact - 1);
    let charged = account(exact);
    let holder = execution.in_realm(&b, |scope| {
        let value = scope.import(&function_in_b)?;
        let callback = scope.callback_object(&value)?;
        assert_eq!(callback.associated_realm().identity(), a_id);
        assert!(matches!(
            scope.retain_callback_object(callback, |bytes| Backing::admit(&short, bytes)),
            Err(Error::ConversionLedger(_))
        ));
        assert_eq!(live(&short), 0);
        let callback = scope.callback_object(&value)?;
        let holder =
            scope.retain_callback_object(callback, |bytes| Backing::admit(&charged, bytes))?;
        assert_eq!(holder.associated_realm().identity(), a_id);
        let opened = scope.open_callback_object(&holder)?;
        assert!(scope.strict_equals(&value, &opened)?);
        assert_eq!(live(&charged), exact);
        Ok(holder)
    })?;
    drop((function, function_in_b, object, object_in_a, a));
    evaluate(&mut execution, &b, "gc()")?;
    execution.memory_pressure();
    assert_eq!(drops.get(), 0);
    execution.in_realm(&b, |scope| {
        let value = scope.open_callback_object(&holder)?;
        let receiver = scope.primitive(Primitive::Undefined)?;
        let actual = scope.call(&value, &receiver, &[])?;
        let expected = scope.primitive(Primitive::Number(41.0))?;
        assert!(scope.strict_equals(&actual, &expected)?);
        drop(holder);
        assert_eq!(
            live(&charged),
            exact,
            "the SDK still owns the persistent handle"
        );
        Ok(())
    })?;
    assert_eq!(
        live(&charged),
        0,
        "the owning boundary must reap the backing"
    );
    evaluate(&mut execution, &b, "gc()")?;
    execution.memory_pressure();
    assert_eq!(drops.get(), 1);
    Ok(())
}

fn proxy_with_traps(
    execution: &mut Execution<'_>,
    realm: &Realm,
    target: &Root,
    traps: &Rc<Cell<usize>>,
) -> Result<Root, Error> {
    let count = traps.clone();
    execution.in_realm(realm, |scope| {
        let target = scope.import(target)?;
        let mark = scope.function(&Callback::new(move |_, _| {
            count.set(count.get() + 1);
            Ok(Completion::Undefined)
        }))?;
        let create = scope.evaluate(&units(
            "(target, mark) => new Proxy(target, {
                get(t, k, r) { mark(); return Reflect.get(t, k, r); },
                getPrototypeOf(t) { mark(); return Reflect.getPrototypeOf(t); },
                apply(t, r, args) { mark(); return Reflect.apply(t, r, args); }
            })",
        ))?;
        let undefined = scope.primitive(Primitive::Undefined)?;
        let proxy = scope.call(&create, &undefined, &[target, mark])?;
        Ok(scope.retain(&proxy))
    })
}

pub(super) fn proxy_objects_use_their_own_associated_realm(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let a = execution.create_realm()?;
    let b = execution.create_realm()?;
    let (a_id, a_function) = execution.in_realm(&a, |scope| {
        let function = scope.evaluate(&units("() => 11"))?;
        Ok((scope.realm_authority()?.identity(), scope.retain(&function)))
    })?;
    let (b_id, b_function, b_object) = execution.in_realm(&b, |scope| {
        let function = scope.evaluate(&units("() => 23"))?;
        let object = scope.evaluate(&units("({answer: 37})"))?;
        Ok((
            scope.realm_authority()?.identity(),
            scope.retain(&function),
            scope.retain(&object),
        ))
    })?;
    assert_ne!(a_id, b_id);
    let traps = Rc::new(Cell::new(0));
    let in_a = proxy_with_traps(&mut execution, &a, &b_function, &traps)?;
    let in_b = proxy_with_traps(&mut execution, &b, &a_function, &traps)?;
    let object_in_a = proxy_with_traps(&mut execution, &a, &b_object, &traps)?;
    let retained_in_b = retain_in(&mut execution, &b, &in_a)?;
    let retained_in_a = retain_in(&mut execution, &a, &in_b)?;
    let object_retained_in_b = retain_in(&mut execution, &b, &object_in_a)?;

    for (retaining, original, retained, expected, callable, result) in [
        (&b, &in_a, &retained_in_b, a_id, true, 23.0),
        (&a, &in_b, &retained_in_a, b_id, true, 11.0),
        (&b, &object_in_a, &object_retained_in_b, a_id, false, 37.0),
    ] {
        execution.in_realm(retaining, |scope| {
            let before = traps.get();
            let proxy = scope.import(retained)?;
            let observed = scope.callback_object(&proxy).unwrap_or_else(|error| {
                panic!("the exact Proxy's creation context must be observable: {error:?}")
            });
            assert_eq!(observed.associated_realm().identity(), expected);
            let charged = account(u64::MAX);
            let holder =
                scope.retain_callback_object(observed, |bytes| Backing::admit(&charged, bytes))?;
            assert_eq!(holder.associated_realm().identity(), expected);
            let opened = scope.open_callback_object(&holder)?;
            let exact = scope.import(original)?;
            assert!(scope.strict_equals(&opened, &exact)?);
            assert_eq!(scope.is_callable(&opened)?, callable);
            assert_eq!(traps.get(), before, "qualification must be trap-free");
            let observed = if callable {
                let undefined = scope.primitive(Primitive::Undefined)?;
                scope.call(&opened, &undefined, &[])?
            } else {
                scope.get(&opened, PropertyKey::String(&units("answer")))?
            };
            let expected = scope.primitive(Primitive::Number(result))?;
            assert!(scope.strict_equals(&observed, &expected)?);
            assert_eq!(
                traps.get(),
                before + 1,
                "the invocation control must reach its trap"
            );
            Ok(())
        })?;
    }
    Ok(())
}

pub(super) fn terminal_cleanup() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let first = account(u64::MAX);
    let second = account(u64::MAX);
    let (release, keep) = execution.in_realm(&realm, |scope| {
        let object = scope.object(ObjectPrototype::Default)?;
        let mut pin = |account| {
            let callback = scope.callback_object(&object)?;
            scope.retain_callback_object(callback, |bytes| Backing::admit(account, bytes))
        };
        Ok::<(RetainedCallbackObject, RetainedCallbackObject), Error>((pin(&first)?, pin(&second)?))
    })?;
    assert!(live(&first) > 0 && live(&second) > 0);
    assert!(matches!(
        execution.in_realm(&realm, |scope| {
            let receiver = scope.primitive(Primitive::Undefined)?;
            let failing = scope.function(&Callback::new(|_, _| Err(Error::IntegerOutOfRange)))?;
            scope.call(&failing, &receiver, &[])?;
            Ok(())
        }),
        Err(Error::IntegerOutOfRange)
    ));
    drop(execution);
    process.release_callback_object(release)?;
    assert_eq!(live(&first), 0);
    assert!(live(&second) > 0);
    drop(process);
    assert_eq!(live(&second), 0);

    let mut other = Process::bootstrap(EngineKind::V8)?;
    let mut execution = other.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        assert!(matches!(
            scope.open_callback_object(&keep),
            Err(Error::ForeignOwner)
        ));
        Ok(())
    })?;
    drop(execution);
    assert!(matches!(
        other.release_callback_object(keep),
        Err(Error::ForeignOwner)
    ));
    Ok(())
}
