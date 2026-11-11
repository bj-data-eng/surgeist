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
        let proxy = scope.evaluate(&units("new Proxy(function(){}, { get() { throw 1 } })"))?;
        assert!(matches!(
            scope.callback_object(&proxy),
            Err(Error::CallbackRealmUnavailable)
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
