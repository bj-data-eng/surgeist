use surgeist_bindings_js::{
    EngineKind, Error, ExceptionPhase, Execution, IntegerKind, IntegerMode, IntegerValue, Process,
    Realm, Scope, Value,
};

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

fn web_idl_integer_conversion_preserves_coercion_and_exact_modulo(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        for (source, kind, expected) in [
            ("128", IntegerKind::Byte, IntegerValue::Byte(-128)),
            ("-1", IntegerKind::Octet, IntegerValue::Octet(255)),
            ("32768", IntegerKind::Short, IntegerValue::Short(i16::MIN)),
            ("65536", IntegerKind::UnsignedShort, IntegerValue::UnsignedShort(0)),
            ("2147483648", IntegerKind::Long, IntegerValue::Long(i32::MIN)),
            ("-1", IntegerKind::UnsignedLong, IntegerValue::UnsignedLong(u32::MAX)),
            ("2 ** 63", IntegerKind::LongLong, IntegerValue::LongLong(i64::MIN)),
            ("-2", IntegerKind::UnsignedLongLong, IntegerValue::UnsignedLongLong(u64::MAX - 1)),
            ("undefined", IntegerKind::Long, IntegerValue::Long(0)),
            ("null", IntegerKind::Long, IntegerValue::Long(0)),
            ("true", IntegerKind::Long, IntegerValue::Long(1)),
        ] {
            let value = scope.evaluate(&units(source))?;
            assert_eq!(scope.integer(&value, kind, IntegerMode::Wrap)?, expected);
        }
        let value = scope.evaluate(&units("globalThis.conversions = 0; ({valueOf() { conversions++; return -2.9 }, toString() { throw 'must not run' }})"))?;
        assert_eq!(scope.integer(&value, IntegerKind::UnsignedLongLong, IntegerMode::Wrap)?, IntegerValue::UnsignedLongLong(u64::MAX - 1));
        let count = scope.evaluate(&units("conversions"))?;
        assert_eq!(scope.to_utf16(&count)?, units("1"));

        for source in ["2 ** 53", "Infinity", "NaN"] {
            let value = scope.evaluate(&units(source))?;
            assert!(matches!(scope.integer(&value, IntegerKind::UnsignedLongLong, IntegerMode::EnforceRange), Err(Error::IntegerOutOfRange)));
        }
        for (source, expected) in [("2.5", 2), ("3.5", 4), ("-0.5", 0)] {
            let value = scope.evaluate(&units(source))?;
            assert_eq!(scope.integer(&value, IntegerKind::Byte, IntegerMode::Clamp)?, IntegerValue::Byte(expected));
        }
        let maximum = scope.evaluate(&units("Infinity"))?;
        assert_eq!(scope.integer(&maximum, IntegerKind::UnsignedLongLong, IntegerMode::Clamp)?, IntegerValue::UnsignedLongLong(9_007_199_254_740_991));

        // ECMAScript ToNumber rejects these; Number(BigInt) is a distinct
        // explicit conversion and must not be substituted by the adapter.
        for source in ["1n", "Symbol('number')"] {
            let value = scope.evaluate(&units(source))?;
            match scope.integer(&value, IntegerKind::Long, IntegerMode::Wrap) {
                Err(Error::Exception(exception)) => {
                    assert_eq!(exception.phase(), ExceptionPhase::Coerce);
                    let thrown = scope.import(exception.value())?;
                    let name = scope.get(&thrown, &units("name"))?;
                    assert_eq!(scope.to_utf16(&name)?, units("TypeError"));
                }
                _ => return Err(Error::BackendUnavailable),
            }
        }
        let value = scope.evaluate(&units("globalThis.coercionFailure = {}; ({valueOf() {throw coercionFailure}})"))?;
        let exception = match scope.integer(&value, IntegerKind::Long, IntegerMode::Wrap) {
            Err(Error::Exception(exception)) => exception,
            _ => return Err(Error::BackendUnavailable),
        };
        let thrown = scope.import(exception.value())?;
        let original = scope.evaluate(&units("coercionFailure"))?;
        assert!(scope.strict_equals(&thrown, &original)?);
        Ok::<_, Error>(())
    })??;
    Ok(())
}

fn web_idl_boolean_is_noncoercive(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        for source in ["undefined", "null", "false", "0", "-0", "NaN", "0n", "''"] {
            let value = scope.evaluate(&units(source))?;
            assert!(!scope.boolean(&value)?, "{source}");
        }
        for source in ["true", "1", "-1", "Infinity", "1n", "'0'", "Symbol()", "[]", "({})", "new Boolean(false)"] {
            let value = scope.evaluate(&units(source))?;
            assert!(scope.boolean(&value)?, "{source}");
        }
        let value = scope.evaluate(&units("globalThis.booleanCoercions = 0; ({valueOf() { booleanCoercions++; throw 1 }, toString() { booleanCoercions++; throw 2 }, [Symbol.toPrimitive]() { booleanCoercions++; throw 3 }})"))?;
        assert!(scope.boolean(&value)?);
        let proxy = scope.evaluate(&units("new Proxy({}, {get() { booleanCoercions++; throw 4 }})"))?;
        assert!(scope.boolean(&proxy)?);
        let count = scope.evaluate(&units("booleanCoercions"))?;
        assert_eq!(scope.to_utf16(&count)?, units("0"));
        Ok::<_, Error>(())
    })??;
    Ok(())
}

fn web_idl_strings_preserve_units_and_validate_bytes(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let value = scope.string(&[0, 0xd800, 0x61, 0xdc00, 0xd83d, 0xde00, 0xdbff])?;
        let dom = scope.dom_string(&value)?;
        assert_eq!(
            dom.as_code_units(),
            [0, 0xd800, 0x61, 0xdc00, 0xd83d, 0xde00, 0xdbff]
        );
        assert_eq!(
            dom.into_code_units(),
            [0, 0xd800, 0x61, 0xdc00, 0xd83d, 0xde00, 0xdbff]
        );
        let usv = scope.usv_string(&value)?;
        assert_eq!(
            usv.as_code_units(),
            [0, 0xfffd, 0x61, 0xfffd, 0xd83d, 0xde00, 0xfffd]
        );
        assert_eq!(
            usv.into_code_units(),
            [0, 0xfffd, 0x61, 0xfffd, 0xd83d, 0xde00, 0xfffd]
        );
        assert!(matches!(
            scope.byte_string(&value),
            Err(Error::ByteStringOutOfRange)
        ));

        let bytes = scope.string(&[0, 0x7f, 0x80, 0xff])?;
        let bytes = scope.byte_string(&bytes)?;
        assert_eq!(bytes.as_bytes(), [0, 0x7f, 0x80, 0xff]);
        assert_eq!(bytes.into_bytes(), [0, 0x7f, 0x80, 0xff]);
        let too_large = scope.string(&[0x100])?;
        assert!(matches!(
            scope.byte_string(&too_large),
            Err(Error::ByteStringOutOfRange)
        ));
        for (source, expected) in [
            ("null", "null"),
            ("undefined", "undefined"),
            ("123n", "123"),
            ("true", "true"),
            ("''", ""),
        ] {
            let value = scope.evaluate(&units(source))?;
            assert_eq!(scope.dom_string(&value)?.as_code_units(), units(expected));
            assert_eq!(scope.usv_string(&value)?.as_code_units(), units(expected));
            assert_eq!(scope.byte_string(&value)?.as_bytes(), expected.as_bytes());
        }
        Ok::<_, Error>(())
    })??;
    Ok(())
}

fn web_idl_floating_conversions_apply_rounding_and_restrictions(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        for (source, expected) in [
            ("1 + 2 ** -24", 1.0),
            ("1 + 3 * 2 ** -24", f32::from_bits(0x3f80_0002)),
            ("2 ** -150", 0.0),
            ("3 * 2 ** -150", f32::from_bits(2)),
        ] {
            let value = scope.evaluate(&units(source))?;
            assert_eq!(scope.float(&value)?, expected);
            assert_eq!(scope.unrestricted_float(&value)?, expected);
        }
        for source in ["2 ** 128 - 2 ** 104", "2 ** 128 - 2 ** 103 - 2 ** 75"] {
            let value = scope.evaluate(&units(source))?;
            assert_eq!(scope.float(&value)?, f32::MAX);
        }
        for source in [
            "2 ** 128 - 2 ** 103",
            "2 ** 128",
            "-(2 ** 128)",
            "Number.MAX_VALUE",
            "NaN",
            "Infinity",
            "-Infinity",
        ] {
            let value = scope.evaluate(&units(source))?;
            assert!(
                matches!(scope.float(&value), Err(Error::FloatOutOfRange)),
                "{source}"
            );
        }
        let finite = scope.evaluate(&units("Number.MAX_VALUE"))?;
        assert_eq!(scope.double(&finite)?, f64::MAX);
        assert_eq!(scope.unrestricted_double(&finite)?, f64::MAX);
        assert_eq!(scope.unrestricted_float(&finite)?, f32::INFINITY);
        for (source, sign) in [("Infinity", 1.0), ("-Infinity", -1.0)] {
            let value = scope.evaluate(&units(source))?;
            assert!(matches!(scope.double(&value), Err(Error::FloatOutOfRange)));
            assert_eq!(scope.unrestricted_double(&value)?, sign * f64::INFINITY);
            assert_eq!(
                scope.unrestricted_float(&value)?,
                sign as f32 * f32::INFINITY
            );
        }
        let nan = scope.evaluate(&units("NaN"))?;
        assert!(matches!(scope.double(&nan), Err(Error::FloatOutOfRange)));
        assert_eq!(scope.unrestricted_float(&nan)?.to_bits(), 0x7fc0_0000);
        assert_eq!(
            scope.unrestricted_double(&nan)?.to_bits(),
            0x7ff8_0000_0000_0000
        );
        for source in ["-0", "-(2 ** -151)"] {
            let value = scope.evaluate(&units(source))?;
            assert_eq!(scope.float(&value)?.to_bits(), 0x8000_0000);
            assert_eq!(scope.unrestricted_float(&value)?.to_bits(), 0x8000_0000);
        }
        let minus_zero = scope.evaluate(&units("-0"))?;
        assert_eq!(scope.double(&minus_zero)?.to_bits(), 0x8000_0000_0000_0000);
        assert_eq!(
            scope.unrestricted_double(&minus_zero)?.to_bits(),
            0x8000_0000_0000_0000
        );
        for (source, expected) in [("null", 0.0), ("true", 1.0), ("'42.25'", 42.25)] {
            let value = scope.evaluate(&units(source))?;
            assert_eq!(scope.float(&value)?, expected as f32);
            assert_eq!(scope.unrestricted_float(&value)?, expected as f32);
            assert_eq!(scope.double(&value)?, expected);
            assert_eq!(scope.unrestricted_double(&value)?, expected);
        }
        Ok::<_, Error>(())
    })??;
    Ok(())
}

#[derive(Clone, Copy)]
enum PrimitiveConversion {
    DomString,
    UsvString,
    ByteString,
    Float,
    UnrestrictedFloat,
    Double,
    UnrestrictedDouble,
}

impl PrimitiveConversion {
    fn coerce<'scope>(self, scope: &mut Scope<'scope>, value: &Value<'scope>) -> Result<(), Error> {
        match self {
            Self::DomString => scope.dom_string(value).map(|_| ()),
            Self::UsvString => scope.usv_string(value).map(|_| ()),
            Self::ByteString => scope.byte_string(value).map(|_| ()),
            Self::Float => scope.float(value).map(|_| ()),
            Self::UnrestrictedFloat => scope.unrestricted_float(value).map(|_| ()),
            Self::Double => scope.double(value).map(|_| ()),
            Self::UnrestrictedDouble => scope.unrestricted_double(value).map(|_| ()),
        }
    }

    fn hint(self) -> &'static str {
        match self {
            Self::DomString | Self::UsvString | Self::ByteString => "string",
            Self::Float | Self::UnrestrictedFloat | Self::Double | Self::UnrestrictedDouble => {
                "number"
            }
        }
    }
}

fn web_idl_primitive_coercion_runs_once_and_preserves_original_throws(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        for conversion in [PrimitiveConversion::DomString, PrimitiveConversion::UsvString, PrimitiveConversion::ByteString, PrimitiveConversion::Float, PrimitiveConversion::UnrestrictedFloat, PrimitiveConversion::Double, PrimitiveConversion::UnrestrictedDouble] {
            let value = scope.evaluate(&units("globalThis.primitiveCalls = 0; globalThis.primitiveHint = ''; ({[Symbol.toPrimitive](hint) { primitiveCalls++; primitiveHint = hint; return '42' }})"))?;
            conversion.coerce(scope, &value)?;
            let count = scope.evaluate(&units("primitiveCalls + ':' + primitiveHint"))?;
            assert_eq!(scope.to_utf16(&count)?, units(&format!("1:{}", conversion.hint())));

            let value = scope.evaluate(&units("globalThis.primitiveCalls = 0; globalThis.primitiveDiagnostics = 0; globalThis.primitiveThrown = {toString() { primitiveDiagnostics++; throw 7 }, get stack() { primitiveDiagnostics++; throw 8 }}; ({[Symbol.toPrimitive]() { primitiveCalls++; throw primitiveThrown }})"))?;
            let exception = match conversion.coerce(scope, &value) {
                Err(Error::Exception(exception)) => exception,
                _ => return Err(Error::BackendUnavailable),
            };
            assert_eq!(exception.phase(), ExceptionPhase::Coerce);
            let thrown = scope.import(exception.value())?;
            let original = scope.evaluate(&units("primitiveThrown"))?;
            assert!(scope.strict_equals(&thrown, &original)?);
            let counts = scope.evaluate(&units("primitiveCalls + ':' + primitiveDiagnostics"))?;
            assert_eq!(scope.to_utf16(&counts)?, units("1:0"));

            // Symbol fails ToString and ToNumber. BigInt succeeds ToString but
            // fails ToNumber, unlike the explicit Number(bigint) operation.
            for source in ["Symbol()", "1n"] {
                if source == "1n" && conversion.hint() == "string" {
                    continue;
                }
                let value = scope.evaluate(&units(source))?;
                let exception = match conversion.coerce(scope, &value) {
                    Err(Error::Exception(exception)) => exception,
                    _ => return Err(Error::BackendUnavailable),
                };
                assert_eq!(exception.phase(), ExceptionPhase::Coerce);
                let thrown = scope.import(exception.value())?;
                let name = scope.get(&thrown, &units("name"))?;
                assert_eq!(scope.to_utf16(&name)?, units("TypeError"));
            }
        }
        Ok::<_, Error>(())
    })??;
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
    web_idl_integer_conversion_preserves_coercion_and_exact_modulo(&mut process)?;
    web_idl_boolean_is_noncoercive(&mut process)?;
    web_idl_strings_preserve_units_and_validate_bytes(&mut process)?;
    web_idl_floating_conversions_apply_rounding_and_restrictions(&mut process)?;
    web_idl_primitive_coercion_runs_once_and_preserves_original_throws(&mut process)?;

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
