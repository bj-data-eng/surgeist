use std::cell::{Cell, RefCell};
use std::rc::Rc;

use surgeist_bindings_js::{
    Callback, Completion, EngineKind, Error, ExceptionPhase, Execution, IntegerKind, IntegerMode,
    IntegerValue, Primitive, Process, Realm, Scope, Value, ValueKind,
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

fn install(
    execution: &mut Execution<'_>,
    realm: &Realm,
    name: &str,
    callback: &Callback,
) -> Result<(), Error> {
    execution.in_realm(realm, |scope| {
        let function = scope.function(callback)?;
        let global = scope.global()?;
        scope.set(&global, &units(name), &function)
    })?
}

struct DropReceipt(Rc<Cell<usize>>);
impl Drop for DropReceipt {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn native_callbacks_preserve_arguments_completions_and_creation_realm(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let a = execution.create_realm()?;
    let b = execution.create_realm()?;
    evaluate(&mut execution, &a, "globalThis.name = 'A'")?;
    evaluate(&mut execution, &b, "globalThis.name = 'B'")?;
    let callback = Callback::new(|scope, arguments| {
        assert_eq!(arguments.len(), 2);
        assert!(arguments.get(2).is_none());
        assert!(!arguments.is_empty());
        let first = arguments.get(0).ok_or(Error::BackendUnavailable)?;
        assert_eq!(scope.to_utf16(first)?, units("undefined"));
        let receiver = scope.get(arguments.receiver(), &units("tag"))?;
        assert_eq!(scope.to_utf16(&receiver)?, units("receiver"));
        let (_, mut values) = arguments.into_parts();
        Ok(Completion::Return(
            values.pop().ok_or(Error::BackendUnavailable)?,
        ))
    });
    install(&mut execution, &a, "echo", &callback)?;
    assert_eq!(
        evaluate(
            &mut execution,
            &a,
            "echo.call({tag:'receiver'}, undefined, 42)"
        )?,
        units("42")
    );
    assert_eq!(
        evaluate(
            &mut execution,
            &a,
            "try { new echo(); false } catch (error) { error instanceof TypeError }"
        )?,
        units("true")
    );
    let omitted = Callback::new(|scope, arguments| {
        let value = scope.string(&units(if arguments.get(0).is_none() {
            "omitted"
        } else {
            "present"
        }))?;
        Ok(Completion::Return(value))
    });
    install(&mut execution, &a, "omitted", &omitted)?;
    assert_eq!(
        evaluate(&mut execution, &a, "omitted() + ':' + omitted(undefined)")?,
        units("omitted:present")
    );
    let normalized = Callback::new(|scope, arguments| {
        let global = scope.global()?;
        let is_global = scope.strict_equals(arguments.receiver(), &global)?;
        let text = if is_global {
            "global".to_owned()
        } else {
            String::from_utf16_lossy(&scope.to_utf16(arguments.receiver())?)
        };
        Ok(Completion::Return(scope.string(&units(&text))?))
    });
    install(&mut execution, &a, "receiver", &normalized)?;
    assert_eq!(
        evaluate(
            &mut execution,
            &a,
            "[receiver.call(undefined), receiver.call(null), receiver.call(7), receiver.call('x'), receiver.call(true)].join(':')"
        )?,
        units("global:global:7:x:true")
    );
    let returns_receiver = Callback::new(|_, arguments| {
        let (receiver, _) = arguments.into_parts();
        Ok(Completion::Return(receiver))
    });
    install(&mut execution, &a, "returnsReceiver", &returns_receiver)?;
    assert_eq!(
        evaluate(
            &mut execution,
            &a,
            "[7, 'x', true].every(value => { const boxed = returnsReceiver.call(value); return typeof boxed === 'object' && boxed.valueOf() === value; })"
        )?,
        units("true")
    );
    {
        let (receiver_function, creation_global) = execution.in_realm(&a, |scope| {
            let global = scope.global()?;
            let function = scope.get(&global, &units("returnsReceiver"))?;
            Ok::<_, Error>((scope.retain(&function), scope.retain(&global)))
        })??;
        execution.in_realm(&b, |scope| {
            let function = scope.import(&receiver_function)?;
            let creation_global = scope.import(&creation_global)?;
            for source in ["undefined", "null"] {
                let receiver = scope.evaluate(&units(source))?;
                let normalized = scope.call(&function, &receiver, &[])?;
                // Equality checks the receiver itself without crossing V8's
                // separate security boundary for foreign global properties.
                assert!(scope.strict_equals(&normalized, &creation_global)?);
            }
            Ok::<_, Error>(())
        })??;
    }
    install(
        &mut execution,
        &a,
        "returnsUndefined",
        &Callback::new(|_, _| Ok(Completion::Undefined)),
    )?;
    assert_eq!(
        evaluate(&mut execution, &a, "returnsUndefined() === undefined")?,
        units("true")
    );
    let drops = Rc::new(Cell::new(0));
    let receipt = DropReceipt(drops.clone());
    let realm_callback = Callback::new(move |scope, _| {
        let _capture = &receipt;
        // Mutation of the guest Reflect property cannot replace the captured
        // setter intrinsic, including after the public creation-realm expires.
        let global = scope.global()?;
        let value = scope.string(&units("set through A"))?;
        scope.set(&global, &units("fromCallback"), &value)?;
        Ok(Completion::Return(
            scope.evaluate(&units("name + ':' + fromCallback"))?,
        ))
    });
    let function = execution.in_realm(&a, |scope| {
        let function = scope.function(&realm_callback)?;
        let global = scope.global()?;
        // Deliberately form function -> data -> context -> global -> function.
        // Only guest heap edges may sustain this otherwise unreachable cycle.
        scope.set(&global, &units("ownedInA"), &function)?;
        Ok::<_, Error>(scope.retain(&function))
    })??;
    execution.in_realm(&b, |scope| {
        let function = scope.import(&function)?;
        let global = scope.global()?;
        scope.set(&global, &units("fromA"), &function)
    })??;
    evaluate(
        &mut execution,
        &a,
        "Reflect.set = () => { throw 'poisoned setter' }",
    )?;
    drop(function);
    drop(realm_callback);
    drop(a);
    execution.memory_pressure();
    assert_eq!(
        evaluate(&mut execution, &b, "fromA()")?,
        units("A:set through A")
    );
    assert_eq!(drops.get(), 0);
    evaluate(&mut execution, &b, "fromA = null")?;
    evaluate(&mut execution, &b, "gc()")?;
    execution.memory_pressure();
    assert_eq!(
        drops.get(),
        1,
        "callback data context/intrinsic edges must not become Rust root cycles"
    );
    Ok(())
}

fn native_callbacks_recurse_and_release_self_cleared_roots(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let custody = Rc::new(RefCell::new(None::<surgeist_bindings_js::Root>));
    let captured = custody.clone();
    let calls = Rc::new(Cell::new(0));
    let captured_calls = calls.clone();
    let drops = Rc::new(Cell::new(0));
    let receipt = DropReceipt(drops.clone());
    let callback = Callback::new(move |scope, arguments| {
        let _capture = &receipt;
        let depth = scope.integer(
            arguments.get(0).ok_or(Error::BackendUnavailable)?,
            IntegerKind::Long,
            IntegerMode::Wrap,
        )?;
        let IntegerValue::Long(depth) = depth else {
            return Err(Error::BackendUnavailable);
        };
        captured_calls.set(captured_calls.get() + 1);
        if depth == 0 {
            captured
                .try_borrow_mut()
                .map_err(|_| Error::RegistryConflict)?
                .take();
            scope.evaluate(&units("recursive = null; gc()"))?;
            return Ok(Completion::Return(scope.string(&units("done"))?));
        }
        let root = captured
            .try_borrow()
            .map_err(|_| Error::RegistryConflict)?
            .clone()
            .ok_or(Error::ExpiredIdentity)?;
        let function = scope.import(&root)?;
        let receiver = scope.global()?;
        let depth = scope.evaluate(&units(&(depth - 1).to_string()))?;
        Ok(Completion::Return(scope.call(
            &function,
            &receiver,
            &[depth],
        )?))
    });
    let root = execution.in_realm(&realm, |scope| {
        let function = scope.function(&callback)?;
        let global = scope.global()?;
        scope.set(&global, &units("recursive"), &function)?;
        Ok::<_, Error>(scope.retain(&function))
    })??;
    custody.replace(Some(root));
    drop(callback);
    assert_eq!(
        evaluate(&mut execution, &realm, "recursive(5)")?,
        units("done")
    );
    assert_eq!(calls.get(), 6);
    assert!(
        custody
            .try_borrow()
            .map_err(|_| Error::RegistryConflict)?
            .is_none()
    );
    evaluate(&mut execution, &realm, "gc()")?;
    execution.memory_pressure();
    assert_eq!(drops.get(), 1);
    Ok(())
}

fn native_callback_guest_throws_remain_exact(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let throws = Callback::new(|scope, arguments| {
        let (_, mut values) = arguments.into_parts();
        if let Some(value) = values.pop() {
            Ok(Completion::Throw(value))
        } else {
            Ok(Completion::Throw(
                scope.type_error(&units("missing argument"))?,
            ))
        }
    });
    install(&mut execution, &realm, "throws", &throws)?;
    let rethrows = Callback::new(|scope, _| {
        scope.evaluate(&units("throw original"))?;
        Ok(Completion::Undefined)
    });
    install(&mut execution, &realm, "rethrows", &rethrows)?;
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "globalThis.original = {}; [throws, rethrows].every(fn => { try { fn(original); return false; } catch (error) { return error === original; } })"
        )?,
        units("true")
    );
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "try { throws(); false } catch (error) { error instanceof TypeError && error.message === 'missing argument' }"
        )?,
        units("true")
    );
    // Rust may handle an operation-local guest exception and continue the
    // callback; it does not leak a pending TryCatch exception into the return.
    let handles = Callback::new(|scope, _| {
        assert!(matches!(
            scope.evaluate(&units("throw original")),
            Err(Error::Exception(_))
        ));
        Ok(Completion::Return(scope.string(&units("recovered"))?))
    });
    install(&mut execution, &realm, "handles", &handles)?;
    assert_eq!(
        evaluate(&mut execution, &realm, "handles()")?,
        units("recovered")
    );
    Ok(())
}

fn native_callback_capture_collection_and_owner_teardown(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let drops = Rc::new(Cell::new(0));
    let receipt = DropReceipt(drops.clone());
    let callback = Callback::new(move |_, _| {
        let _capture = &receipt;
        Ok(Completion::Undefined)
    });
    install(&mut execution, &realm, "unreachable", &callback)?;
    drop(callback);
    evaluate(&mut execution, &realm, "unreachable = null")?;
    evaluate(&mut execution, &realm, "gc()")?;
    execution.memory_pressure();
    assert_eq!(drops.get(), 1);
    Ok(())
}

fn host_refusals_are_sticky_and_prevent_native_effects() -> Result<(), Error> {
    for source in [
        "try { fail() } catch (_) {} later()",
        "coerce({valueOf() { try { fail() } catch (_) {} return 1; }})",
        "firstThenReturn()",
        "firstThenThrow()",
        "firstThenError()",
    ] {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let mut execution = process.execution();
        let realm = execution.create_realm()?;
        let effects = Rc::new(Cell::new(0));
        install(
            &mut execution,
            &realm,
            "fail",
            &Callback::new(|_, _| Err(Error::IntegerOutOfRange)),
        )?;
        let effect = effects.clone();
        install(
            &mut execution,
            &realm,
            "later",
            &Callback::new(move |scope, _| {
                scope.check_active()?;
                effect.set(effect.get() + 1);
                Ok(Completion::Undefined)
            }),
        )?;
        let effect = effects.clone();
        install(
            &mut execution,
            &realm,
            "coerce",
            &Callback::new(move |scope, arguments| {
                let value = arguments.get(0).ok_or(Error::BackendUnavailable)?;
                let _refused = scope.integer(value, IntegerKind::Long, IntegerMode::Wrap);
                if scope.check_active().is_ok() {
                    effect.set(effect.get() + 1);
                }
                Ok(Completion::Undefined)
            }),
        )?;
        install(
            &mut execution,
            &realm,
            "firstThenReturn",
            &Callback::new(|scope, _| {
                let _refused = scope.evaluate(&units("try { fail() } catch (_) {}"));
                Ok(Completion::Undefined)
            }),
        )?;
        install(
            &mut execution,
            &realm,
            "firstThenThrow",
            &Callback::new(|scope, _| {
                let thrown = scope.string(&units("later throw"))?;
                let _refused = scope.evaluate(&units("try { fail() } catch (_) {}"));
                Ok(Completion::Throw(thrown))
            }),
        )?;
        install(
            &mut execution,
            &realm,
            "firstThenError",
            &Callback::new(|scope, _| {
                let _refused = scope.evaluate(&units("try { fail() } catch (_) {}"));
                Err(Error::Allocation)
            }),
        )?;
        assert!(
            matches!(
                evaluate(&mut execution, &realm, source),
                Err(Error::IntegerOutOfRange)
            ),
            "{source}"
        );
        assert_eq!(effects.get(), 0);
        assert!(matches!(
            evaluate(&mut execution, &realm, "later()"),
            Err(Error::IntegerOutOfRange)
        ));
        assert!(matches!(
            execution.checkpoint(),
            Err(Error::IntegerOutOfRange)
        ));
        assert_eq!(effects.get(), 0);
    }
    let drops = Rc::new(Cell::new(0));
    {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let mut execution = process.execution();
        let realm = execution.create_realm()?;
        let receipt = DropReceipt(drops.clone());
        install(
            &mut execution,
            &realm,
            "retained",
            &Callback::new(move |_, _| {
                let _capture = &receipt;
                Ok(Completion::Undefined)
            }),
        )?;
        evaluate(&mut execution, &realm, "Promise.resolve().then(retained)")?;
        assert_eq!(drops.get(), 0);
    }
    assert_eq!(drops.get(), 1);
    struct PanicReceipt(std::sync::Arc<std::sync::atomic::AtomicUsize>);
    impl Drop for PanicReceipt {
        fn drop(&mut self) {
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    let payload_drops = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let mut execution = process.execution();
        let realm = execution.create_realm()?;
        let captured_payload_drops = payload_drops.clone();
        install(
            &mut execution,
            &realm,
            "panics",
            &Callback::new(move |_, _| {
                std::panic::panic_any(PanicReceipt(captured_payload_drops.clone()))
            }),
        )?;
        assert!(matches!(
            evaluate(&mut execution, &realm, "try { panics() } catch (_) {}"),
            Err(Error::CallbackPanicked)
        ));
        assert!(matches!(
            execution.create_realm(),
            Err(Error::CallbackPanicked)
        ));
        assert_eq!(payload_drops.load(std::sync::atomic::Ordering::Relaxed), 0);
    }
    assert_eq!(payload_drops.load(std::sync::atomic::Ordering::Relaxed), 1);
    Ok(())
}

fn scalar_construction_preserves_javascript_values(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let undefined = scope.primitive(Primitive::Undefined)?;
        let null = scope.primitive(Primitive::Null)?;
        assert_eq!(scope.kind(&undefined)?, ValueKind::Undefined);
        assert_eq!(scope.kind(&null)?, ValueKind::Null);
        assert!(!scope.strict_equals(&undefined, &null)?);
        assert!(!scope.is_callable(&undefined)?);
        assert!(!scope.is_callable(&null)?);
        assert!(scope.unrestricted_double(&undefined)?.is_nan());
        assert_eq!(scope.double(&null)?, 0.0);
        for truth in [false, true] {
            let value = scope.primitive(Primitive::Boolean(truth))?;
            assert_eq!(scope.kind(&value)?, ValueKind::Boolean);
            assert_eq!(scope.boolean(&value)?, truth);
            assert_eq!(scope.double(&value)?, if truth { 1.0 } else { 0.0 });
            assert!(!scope.is_callable(&value)?);
        }
        for number in [
            0.0,
            -0.0,
            1.25,
            -42.5,
            f64::MAX,
            -f64::MAX,
            f64::MIN_POSITIVE,
            f64::from_bits(1),
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            let value = scope.primitive(Primitive::Number(number))?;
            assert_eq!(scope.kind(&value)?, ValueKind::Number);
            assert_eq!(
                scope.unrestricted_double(&value)?.to_bits(),
                number.to_bits()
            );
            assert_eq!(scope.boolean(&value)?, number != 0.0);
            assert!(!scope.is_callable(&value)?);
        }
        // ECMAScript exposes one NaN value, not host NaN payload identities.
        for number in [
            f64::NAN,
            f64::from_bits(0x7ff0_0000_0000_0001),
            f64::from_bits(0xffff_ffff_ffff_ffff),
        ] {
            let value = scope.primitive(Primitive::Number(number))?;
            assert_eq!(scope.kind(&value)?, ValueKind::Number);
            assert!(scope.unrestricted_double(&value)?.is_nan());
            assert!(!scope.boolean(&value)?);
        }
        Ok::<_, Error>(())
    })??;
    Ok(())
}

fn value_classification_uses_language_types_without_guest_hooks(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        for (source, kind) in [
            ("undefined", ValueKind::Undefined),
            ("null", ValueKind::Null),
            ("false", ValueKind::Boolean),
            ("NaN", ValueKind::Number),
            ("123456789012345678901234567890n", ValueKind::BigInt),
            ("'text'", ValueKind::String),
            ("Symbol('tag')", ValueKind::Symbol),
            ("({})", ValueKind::Object),
            ("[]", ValueKind::Object),
            ("new Number(7)", ValueKind::Object),
            ("new Boolean(false)", ValueKind::Object),
            ("new String('text')", ValueKind::Object),
        ] {
            let value = scope.evaluate(&units(source))?;
            assert_eq!(scope.kind(&value)?, kind, "{source}");
            assert!(!scope.is_callable(&value)?, "{source}");
        }
        let string = scope.string(&[0xd800, 0])?;
        assert_eq!(scope.kind(&string)?, ValueKind::String);
        assert!(!scope.is_callable(&string)?);
        scope.evaluate(&units("globalThis.classificationHooks = 0"))?;
        for (source, callable) in [
            ("({ [Symbol.toPrimitive]() { classificationHooks++; throw 1 }, valueOf() { classificationHooks++; throw 2 }, toString() { classificationHooks++; throw 3 } })", false),
            ("new Proxy({}, { get() { classificationHooks++; throw 4 }, getPrototypeOf() { classificationHooks++; throw 5 } })", false),
            ("new Proxy(function(){}, { get() { classificationHooks++; throw 6 }, getPrototypeOf() { classificationHooks++; throw 7 } })", true),
        ] {
            let value = scope.evaluate(&units(source))?;
            assert_eq!(scope.kind(&value)?, ValueKind::Object);
            assert_eq!(scope.is_callable(&value)?, callable);
        }
        let count = scope.evaluate(&units("classificationHooks"))?;
        assert_eq!(scope.double(&count)?, 0.0);
        Ok::<_, Error>(())
    })??;
    Ok(())
}

fn callable_classification_includes_classes_and_revoked_callable_proxies(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let receiver = scope.primitive(Primitive::Undefined)?;
        for source in ["(function ordinary() { return 42 })", "(() => 42)", "(function() { return 42 }).bind(null)", "new Proxy(() => 42, {})"] {
            let function = scope.evaluate(&units(source))?;
            assert_eq!(scope.kind(&function)?, ValueKind::Object, "{source}");
            assert!(scope.is_callable(&function)?, "{source}");
            let result = scope.call(&function, &receiver, &[])?;
            assert_eq!(scope.double(&result)?, 42.0);
        }
        for source in ["(class Constructor {})", "(() => { const pair = Proxy.revocable(() => 42, {}); pair.revoke(); return pair.proxy })()"] {
            let function = scope.evaluate(&units(source))?;
            assert_eq!(scope.kind(&function)?, ValueKind::Object);
            assert!(scope.is_callable(&function)?);
            let exception = match scope.call(&function, &receiver, &[]) {
                Err(Error::Exception(exception)) => exception,
                _ => return Err(Error::BackendUnavailable),
            };
            assert_eq!(exception.phase(), ExceptionPhase::Invoke);
            let thrown = scope.import(exception.value())?;
            let name = scope.get(&thrown, &units("name"))?;
            assert_eq!(scope.to_utf16(&name)?, units("TypeError"));
            // Revocation and an unsuccessful call do not remove [[Call]].
            assert!(scope.is_callable(&function)?);
        }
        for source in ["new Proxy({}, {})", "(() => { const pair = Proxy.revocable({}, {}); pair.revoke(); return pair.proxy })()"] {
            let object = scope.evaluate(&units(source))?;
            assert_eq!(scope.kind(&object)?, ValueKind::Object);
            assert!(!scope.is_callable(&object)?);
            assert!(matches!(scope.call(&object, &receiver, &[]), Err(Error::ExpectedFunction)));
        }
        Ok::<_, Error>(())
    })??;
    Ok(())
}

fn scalar_and_classification_operations_work_in_native_callbacks(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let omitted = Callback::new(|scope, arguments| {
        let omitted = match arguments.get(0) {
            None => true,
            Some(value) => {
                assert_eq!(scope.kind(value)?, ValueKind::Undefined);
                false
            }
        };
        Ok(Completion::Return(
            scope.primitive(Primitive::Boolean(omitted))?,
        ))
    });
    let classify = Callback::new(|scope, arguments| {
        let expected = [
            ValueKind::Undefined,
            ValueKind::Null,
            ValueKind::Boolean,
            ValueKind::Number,
            ValueKind::BigInt,
            ValueKind::String,
            ValueKind::Symbol,
            ValueKind::Object,
            ValueKind::Object,
        ];
        assert_eq!(arguments.len(), expected.len());
        for (index, expected) in expected.into_iter().enumerate() {
            let value = arguments.get(index).ok_or(Error::BackendUnavailable)?;
            assert_eq!(scope.kind(value)?, expected);
            assert_eq!(scope.is_callable(value)?, index == 8);
        }
        Ok(Completion::Return(
            scope.primitive(Primitive::Number(42.5))?,
        ))
    });
    execution.in_realm(&realm, |scope| {
        let receiver = scope.primitive(Primitive::Undefined)?;
        let omitted = scope.function(&omitted)?;
        assert_eq!(scope.kind(&omitted)?, ValueKind::Object);
        assert!(scope.is_callable(&omitted)?);
        let result = scope.call(&omitted, &receiver, &[])?;
        assert_eq!(scope.kind(&result)?, ValueKind::Boolean);
        assert!(scope.boolean(&result)?);
        let undefined = scope.primitive(Primitive::Undefined)?;
        let result = scope.call(&omitted, &receiver, &[undefined])?;
        assert!(!scope.boolean(&result)?);

        let classify = scope.function(&classify)?;
        let arguments = [
            scope.primitive(Primitive::Undefined)?,
            scope.primitive(Primitive::Null)?,
            scope.primitive(Primitive::Boolean(true))?,
            scope.primitive(Primitive::Number(-0.0))?,
            scope.evaluate(&units("1n"))?,
            scope.string(&[0xd800, 0])?,
            scope.evaluate(&units("Symbol()"))?,
            scope.global()?,
            scope.function(&Callback::new(|_, _| Ok(Completion::Undefined)))?,
        ];
        let result = scope.call(&classify, &receiver, &arguments)?;
        assert_eq!(scope.kind(&result)?, ValueKind::Number);
        assert_eq!(scope.double(&result)?, 42.5);
        Ok::<_, Error>(())
    })??;
    Ok(())
}

fn scalar_operations_preserve_the_first_host_failure() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    install(
        &mut execution,
        &realm,
        "failScalar",
        &Callback::new(|_, _| Err(Error::Allocation)),
    )?;
    let checked = Rc::new(Cell::new(false));
    let recorded = checked.clone();
    install(
        &mut execution,
        &realm,
        "probeScalar",
        &Callback::new(move |scope, arguments| {
            let value = arguments.get(0).ok_or(Error::BackendUnavailable)?;
            let global = scope.global()?;
            let fail = scope.get(&global, &units("failScalar"))?;
            let receiver = scope.primitive(Primitive::Undefined)?;
            assert!(matches!(
                scope.call(&fail, &receiver, &[]),
                Err(Error::Allocation)
            ));
            assert!(matches!(
                scope.primitive(Primitive::Number(1.0)),
                Err(Error::Allocation)
            ));
            assert!(matches!(scope.kind(value), Err(Error::Allocation)));
            assert!(matches!(scope.is_callable(value), Err(Error::Allocation)));
            // Test-only receipt proves every refusal was observed inside the
            // callback; its proposed completion must still lose to the first error.
            recorded.set(true);
            Ok(Completion::Undefined)
        }),
    )?;
    assert!(matches!(
        evaluate(&mut execution, &realm, "probeScalar(7)"),
        Err(Error::Allocation)
    ));
    assert!(checked.get());
    Ok(())
}

fn main() -> Result<(), Error> {
    v8::V8::set_flags_from_string("--expose-gc");
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
    native_callbacks_preserve_arguments_completions_and_creation_realm(&mut process)?;
    native_callbacks_recurse_and_release_self_cleared_roots(&mut process)?;
    native_callback_guest_throws_remain_exact(&mut process)?;
    native_callback_capture_collection_and_owner_teardown(&mut process)?;
    scalar_construction_preserves_javascript_values(&mut process)?;
    value_classification_uses_language_types_without_guest_hooks(&mut process)?;
    callable_classification_includes_classes_and_revoked_callable_proxies(&mut process)?;
    scalar_and_classification_operations_work_in_native_callbacks(&mut process)?;

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
    drop(execution);
    drop(next);
    host_refusals_are_sticky_and_prevent_native_effects()?;
    scalar_operations_preserve_the_first_host_failure()?;
    println!(
        "execution ownership, realms, roots, UTF-16, exceptions, properties, calls, jobs and native callbacks passed"
    );
    Ok(())
}
