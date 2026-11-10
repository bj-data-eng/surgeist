use surgeist_bindings_js::{EngineKind, Error, ExceptionPhase, Execution, Process, Realm};

fn units(source: &str) -> Vec<u16> {
    source.encode_utf16().collect()
}

fn evaluate(execution: &mut Execution<'_>, realm: &Realm, source: &str) -> Result<Vec<u16>, Error> {
    execution.in_realm(realm, |scope| {
        let value = scope.evaluate(&units(source))?;
        scope.to_utf16(&value)
    })?
}

fn realm_and_root_contracts(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let a = execution.create_realm()?;
    let b = execution.create_realm()?;
    assert_eq!(
        evaluate(&mut execution, &a, "globalThis.name = 'A'; name")?,
        units("A")
    );
    assert_eq!(
        evaluate(&mut execution, &b, "globalThis.name = 'B'; name")?,
        units("B")
    );
    assert_eq!(evaluate(&mut execution, &a, "name")?, units("A"));

    let function = execution.in_realm(&a, |scope| {
        let function = scope.evaluate(&units("(argument => name + ':' + argument)"))?;
        Ok::<_, Error>(scope.retain(&function))
    })??;
    execution.memory_pressure();
    execution.in_realm(&b, |scope| {
        let function = scope.import(&function)?;
        let receiver = scope.global()?;
        let argument = scope.string(&units("from B"))?;
        let value = scope.call(&function, &receiver, &[argument])?;
        assert_eq!(scope.to_utf16(&value)?, units("A:from B"));
        Ok::<_, Error>(())
    })??;

    let object = execution.in_realm(&a, |scope| {
        let object = scope.evaluate(&units("globalThis.retained = {count: 1}"))?;
        Ok::<_, Error>(scope.retain(&object))
    })??;
    execution.memory_pressure();
    execution.in_realm(&a, |scope| {
        let root = scope.import(&object)?;
        let current = scope.evaluate(&units("retained.count++; retained"))?;
        assert!(scope.strict_equals(&root, &current)?);
        let count = scope.get(&root, &units("count"))?;
        assert_eq!(scope.to_utf16(&count)?, units("2"));
        let global = scope.global()?;
        scope.set(&global, &units("fromRust"), &root)?;
        Ok::<_, Error>(())
    })??;
    assert_eq!(
        evaluate(&mut execution, &a, "fromRust === retained")?,
        units("true")
    );

    // Tokens are independent of stack creation order. The remaining realm and
    // root stay usable when an earlier/later peer token is dropped.
    drop(a);
    execution.memory_pressure();
    assert_eq!(evaluate(&mut execution, &b, "name")?, units("B"));
    execution.in_realm(&b, |scope| {
        let object = scope.import(&object)?;
        let count = scope.get(&object, &units("count"))?;
        assert_eq!(scope.to_utf16(&count)?, units("2"));
        Ok::<_, Error>(())
    })??;
    Ok(())
}

fn utf16_is_code_unit_exact(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let original = [0, 0xd800, 0x61, 0xdc00, 0xd83d, 0xde00];
        let value = scope.string(&original)?;
        assert_eq!(scope.to_utf16(&value)?, original);
        let global = scope.global()?;
        let name = [0xd800, 0, 0x70];
        scope.set(&global, &name, &value)?;
        let read = scope.get(&global, &name)?;
        assert_eq!(scope.to_utf16(&read)?, original);
        let source = [0x27, 0xd800, 0x27];
        let value = scope.evaluate(&source)?;
        assert_eq!(scope.to_utf16(&value)?, [0xd800]);
        Ok::<_, Error>(())
    })??;
    Ok(())
}

fn exceptions_preserve_values_without_diagnostics(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let exception = execution.in_realm(&realm, |scope| {
        match scope.evaluate(&units("globalThis.diagnostics = 0; globalThis.thrown = { toString() { diagnostics++; throw 99 }, get stack() { diagnostics++; throw 98 } }; throw thrown")) {
            Err(Error::Exception(exception)) => Ok(exception),
            _ => Err(Error::BackendUnavailable),
        }
    })??;
    assert_eq!(exception.phase(), ExceptionPhase::Evaluate);
    assert_eq!(evaluate(&mut execution, &realm, "diagnostics")?, units("0"));
    execution.in_realm(&realm, |scope| {
        let caught = scope.import(exception.value())?;
        let original = scope.evaluate(&units("thrown"))?;
        assert!(scope.strict_equals(&caught, &original)?);
        let coercion = match scope.to_utf16(&caught) {
            Err(Error::Exception(exception)) => exception,
            _ => return Err(Error::BackendUnavailable),
        };
        assert_eq!(coercion.phase(), ExceptionPhase::Coerce);
        let value = scope.import(coercion.value())?;
        assert_eq!(scope.to_utf16(&value)?, units("99"));
        Ok::<_, Error>(())
    })??;
    assert_eq!(evaluate(&mut execution, &realm, "diagnostics")?, units("1"));
    execution.in_realm(&realm, |scope| {
        match scope.evaluate(&units("let = ;")) {
            Err(Error::Exception(exception)) => {
                assert_eq!(exception.phase(), ExceptionPhase::Compile)
            }
            _ => return Err(Error::BackendUnavailable),
        }
        for source in ["throw undefined", "throw null", "throw 7"] {
            match scope.evaluate(&units(source)) {
                Err(Error::Exception(exception)) => {
                    assert_eq!(exception.phase(), ExceptionPhase::Evaluate)
                }
                _ => return Err(Error::BackendUnavailable),
            }
        }
        let value = scope.evaluate(&units("6 * 7"))?;
        assert_eq!(scope.to_utf16(&value)?, units("42"));
        Ok::<_, Error>(())
    })??;
    Ok(())
}

fn properties_and_calls_refuse_invalid_values(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let primitive = scope.evaluate(&units("3"))?;
        assert!(matches!(
            scope.get(&primitive, &units("x")),
            Err(Error::ExpectedObject)
        ));
        assert!(matches!(
            scope.call(&primitive, &primitive, &[]),
            Err(Error::ExpectedFunction)
        ));
        let object = scope.evaluate(&units("Object.freeze({x: 1})"))?;
        scope.evaluate(&units("Reflect.set = () => true"))?;
        assert!(matches!(
            scope.set(&object, &units("x"), &primitive),
            Err(Error::PropertyRejected)
        ));
        let getter = scope.evaluate(&units("({get x() { throw 17 }})"))?;
        match scope.get(&getter, &units("x")) {
            Err(Error::Exception(exception)) => {
                assert_eq!(exception.phase(), ExceptionPhase::PropertyRead)
            }
            _ => return Err(Error::BackendUnavailable),
        }
        let setter = scope.evaluate(&units("({set x(value) { throw value }})"))?;
        match scope.set(&setter, &units("x"), &primitive) {
            Err(Error::Exception(exception)) => {
                assert_eq!(exception.phase(), ExceptionPhase::PropertyWrite)
            }
            _ => return Err(Error::BackendUnavailable),
        }
        let function = scope.evaluate(&units("function fail() {throw 19}; fail"))?;
        match scope.call(&function, &primitive, &[]) {
            Err(Error::Exception(exception)) => {
                assert_eq!(exception.phase(), ExceptionPhase::Invoke)
            }
            _ => return Err(Error::BackendUnavailable),
        }
        Ok::<_, Error>(())
    })??;
    Ok(())
}

fn checkpoints_are_explicit_and_shared(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let a = execution.create_realm()?;
    let b = execution.create_realm()?;
    evaluate(
        &mut execution,
        &a,
        "globalThis.order = ['script']; Promise.resolve().then(() => {order.push('A'); Promise.resolve().then(() => order.push('nested'))}); 0",
    )?;
    evaluate(
        &mut execution,
        &b,
        "globalThis.order = []; Promise.resolve().then(() => order.push('B')); 0",
    )?;
    assert_eq!(
        evaluate(&mut execution, &a, "order.join(',')")?,
        units("script")
    );
    assert_eq!(evaluate(&mut execution, &b, "order.join(',')")?, units(""));
    execution.checkpoint()?;
    assert_eq!(
        evaluate(&mut execution, &a, "order.join(',')")?,
        units("script,A,nested")
    );
    assert_eq!(evaluate(&mut execution, &b, "order.join(',')")?, units("B"));
    Ok(())
}

fn main() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    assert!(matches!(
        Process::bootstrap(EngineKind::V8),
        Err(Error::OwnerAlreadyActive)
    ));
    let foreign_thread_refused = std::thread::spawn(|| {
        matches!(Process::bootstrap(EngineKind::V8), Err(Error::WrongThread))
    })
    .join()
    .expect("bootstrap probe thread");
    assert!(foreign_thread_refused);
    realm_and_root_contracts(&mut process)?;
    utf16_is_code_unit_exact(&mut process)?;
    exceptions_preserve_values_without_diagnostics(&mut process)?;
    properties_and_calls_refuse_invalid_values(&mut process)?;
    checkpoints_are_explicit_and_shared(&mut process)?;

    let (old_realm, old_value) = {
        let mut execution = process.execution();
        let realm = execution.create_realm()?;
        let value = execution.in_realm(&realm, |scope| {
            let value = scope.evaluate(&units("Promise.resolve().then(() => { throw new Error('must not run at teardown') }); ({alive: true})"))?;
            Ok::<_, Error>(scope.retain(&value))
        })??;
        (realm, value)
    };
    drop(process);
    let mut next = Process::bootstrap(EngineKind::V8)?;
    let mut execution = next.execution();
    assert!(matches!(
        execution.in_realm(&old_realm, |_| ()),
        Err(Error::ForeignOwner)
    ));
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        assert!(matches!(scope.import(&old_value), Err(Error::ForeignOwner)));
    })?;
    // Forgetting a temporary borrow cannot lose ownership of the isolate.
    std::mem::forget(execution);
    let mut execution = next.execution();
    assert_eq!(evaluate(&mut execution, &realm, "42")?, units("42"));
    println!(
        "execution ownership, realms, roots, UTF-16, exceptions, properties, calls and jobs passed"
    );
    Ok(())
}
