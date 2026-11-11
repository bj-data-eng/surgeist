#[path = "execution/dom_exception.rs"]
mod dom_exception;
#[path = "execution/host.rs"]
mod host;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use surgeist_bindings_js::{ConstructorCallback, ConstructorCompletion, FunctionMetadata};
use surgeist_bindings_js::{
    Intrinsic, ObjectPrototype, PropertyDescriptor, PropertyKey, WellKnownSymbol,
};

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
    })
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
    })?;
    execution.memory_pressure();
    execution.in_realm(&b, |scope| {
        let function = scope.import(&function)?;
        let receiver = scope.global()?;
        let argument = scope.string(&units("from B"))?;
        let value = scope.call(&function, &receiver, &[argument])?;
        assert_eq!(scope.to_utf16(&value)?, units("A:from B"));
        Ok::<_, Error>(())
    })?;

    let object = execution.in_realm(&a, |scope| {
        let object = scope.evaluate(&units("globalThis.retained = {count: 1}"))?;
        Ok::<_, Error>(scope.retain(&object))
    })?;
    execution.memory_pressure();
    execution.in_realm(&a, |scope| {
        let root = scope.import(&object)?;
        let current = scope.evaluate(&units("retained.count++; retained"))?;
        assert!(scope.strict_equals(&root, &current)?);
        let count = scope.get(&root, PropertyKey::String(&units("count")))?;
        assert_eq!(scope.to_utf16(&count)?, units("2"));
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("fromRust")), &root)?;
        Ok::<_, Error>(())
    })?;
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
        let count = scope.get(&object, PropertyKey::String(&units("count")))?;
        assert_eq!(scope.to_utf16(&count)?, units("2"));
        Ok::<_, Error>(())
    })?;
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
        scope.set(&global, PropertyKey::String(&name), &value)?;
        let read = scope.get(&global, PropertyKey::String(&name))?;
        assert_eq!(scope.to_utf16(&read)?, original);
        let source = [0x27, 0xd800, 0x27];
        let value = scope.evaluate(&source)?;
        assert_eq!(scope.to_utf16(&value)?, [0xd800]);
        Ok::<_, Error>(())
    })?;
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
    })?;
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
    })?;
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
    })?;
    Ok(())
}

fn properties_and_calls_refuse_invalid_values(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let primitive = scope.evaluate(&units("3"))?;
        assert!(matches!(
            scope.get(&primitive, PropertyKey::String(&units("x"))),
            Err(Error::ExpectedObject)
        ));
        assert!(matches!(
            scope.call(&primitive, &primitive, &[]),
            Err(Error::ExpectedFunction)
        ));
        let object = scope.evaluate(&units("Object.freeze({x: 1})"))?;
        scope.evaluate(&units("Reflect.set = () => true"))?;
        assert!(matches!(
            scope.set(&object, PropertyKey::String(&units("x")), &primitive),
            Err(Error::PropertyRejected)
        ));
        let getter = scope.evaluate(&units("({get x() { throw 17 }})"))?;
        match scope.get(&getter, PropertyKey::String(&units("x"))) {
            Err(Error::Exception(exception)) => {
                assert_eq!(exception.phase(), ExceptionPhase::PropertyRead)
            }
            _ => return Err(Error::BackendUnavailable),
        }
        let setter = scope.evaluate(&units("({set x(value) { throw value }})"))?;
        match scope.set(&setter, PropertyKey::String(&units("x")), &primitive) {
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
    })?;
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
                    let name = scope.get(&thrown, PropertyKey::String(&units("name")))?;
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
    })?;
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
    })?;
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
    })?;
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
    })?;
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
                let name = scope.get(&thrown, PropertyKey::String(&units("name")))?;
                assert_eq!(scope.to_utf16(&name)?, units("TypeError"));
            }
        }
        Ok::<_, Error>(())
    })?;
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
        scope.set(&global, PropertyKey::String(&units(name)), &function)
    })
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
        let receiver = scope.get(arguments.receiver(), PropertyKey::String(&units("tag")))?;
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
            let function = scope.get(&global, PropertyKey::String(&units("returnsReceiver")))?;
            Ok::<_, Error>((scope.retain(&function), scope.retain(&global)))
        })?;
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
        })?;
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
        scope.set(&global, PropertyKey::String(&units("fromCallback")), &value)?;
        Ok(Completion::Return(
            scope.evaluate(&units("name + ':' + fromCallback"))?,
        ))
    });
    let function = execution.in_realm(&a, |scope| {
        let function = scope.function(&realm_callback)?;
        let global = scope.global()?;
        // Deliberately form function -> data -> context -> global -> function.
        // Only guest heap edges may sustain this otherwise unreachable cycle.
        scope.set(&global, PropertyKey::String(&units("ownedInA")), &function)?;
        Ok::<_, Error>(scope.retain(&function))
    })?;
    execution.in_realm(&b, |scope| {
        let function = scope.import(&function)?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("fromA")), &function)
    })?;
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
        scope.set(&global, PropertyKey::String(&units("recursive")), &function)?;
        Ok::<_, Error>(scope.retain(&function))
    })?;
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
    })?;
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
    })?;
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
            let name = scope.get(&thrown, PropertyKey::String(&units("name")))?;
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
    })?;
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
    })?;
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
            let fail = scope.get(&global, PropertyKey::String(&units("failScalar")))?;
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

fn property_keys_are_exact_and_never_coerce(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let object = scope.object(ObjectPrototype::Null)?;
        let name = [0xd800, 0, 0xdc00];
        let key = scope.string(&name)?;
        let value = scope.string(&units("value"))?;
        scope.set(&object, PropertyKey::String(&name), &value)?;
        let read = scope.get(&object, PropertyKey::Value(&key))?;
        assert!(scope.strict_equals(&read, &value)?);
        let other = scope.string(&units("replacement"))?;
        scope.set(&object, PropertyKey::Value(&key), &other)?;
        let read = scope.get(&object, PropertyKey::String(&name))?;
        assert!(scope.strict_equals(&read, &other)?);
        for (symbol, source) in [(WellKnownSymbol::Iterator, "Symbol.iterator"), (WellKnownSymbol::ToStringTag, "Symbol.toStringTag"), (WellKnownSymbol::Unscopables, "Symbol.unscopables")] {
            let symbol = scope.well_known_symbol(symbol)?;
            let original = scope.evaluate(&units(source))?;
            assert!(scope.strict_equals(&symbol, &original)?);
            scope.set(&object, PropertyKey::Value(&symbol), &value)?;
            let read = scope.get(&object, PropertyKey::Value(&original))?;
            assert!(scope.strict_equals(&read, &value)?);
        }
        let tag = scope.well_known_symbol(WellKnownSymbol::ToStringTag)?;
        scope.define_property(&object, PropertyKey::Value(&tag), PropertyDescriptor::Data { value: &value, writable: false, enumerable: false, configurable: true })?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("symbolKeyed")), &object)?;
        let exact = scope.evaluate(&units("Object.prototype.toString.call(symbolKeyed) === '[object value]' && Object.getOwnPropertyDescriptor(symbolKeyed, Symbol.toStringTag).writable === false"))?;
        assert!(scope.boolean(&exact)?);
        // An invalid key must not invoke either ToPropertyKey hooks or target
        // traps. Refusal is a typed Rust result, with no guest TypeError.
        let target = scope.evaluate(&units("globalThis.keyEffects = 0; new Proxy({}, {get() {keyEffects++; return 1}, set() {keyEffects++; return true}, defineProperty() {keyEffects++; return true}})"))?;
        for source in ["0", "null", "undefined", "true", "1n", "({[Symbol.toPrimitive]() { keyEffects++; throw 7; }})"] {
            let key = scope.evaluate(&units(source))?;
            assert!(matches!(scope.get(&target, PropertyKey::Value(&key)), Err(Error::ExpectedPropertyKey)));
            assert!(matches!(scope.set(&target, PropertyKey::Value(&key), &value), Err(Error::ExpectedPropertyKey)));
            assert!(matches!(scope.define_property(&target, PropertyKey::Value(&key), PropertyDescriptor::Data { value: &value, writable: true, enumerable: true, configurable: true }), Err(Error::ExpectedPropertyKey)));
        }
        let effects = scope.evaluate(&units("keyEffects"))?;
        assert_eq!(scope.to_utf16(&effects)?, units("0"));
        let original = scope.well_known_symbol(WellKnownSymbol::Iterator)?;
        scope.evaluate(&units("globalThis.Symbol = {iterator: 'poisoned'}"))?;
        let actual = scope.well_known_symbol(WellKnownSymbol::Iterator)?;
        assert!(scope.strict_equals(&actual, &original)?);
        Ok::<_, Error>(())
    })?;
    Ok(())
}

fn descriptors_preserve_attributes_reentry_and_original_throws(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let object = scope.object(ObjectPrototype::Default)?;
        let value = scope.object(ObjectPrototype::Null)?;
        scope.define_property(&object, PropertyKey::String(&units("data")), PropertyDescriptor::Data { value: &value, writable: false, enumerable: false, configurable: false })?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("described")), &object)?;
        scope.set(&global, PropertyKey::String(&units("describedValue")), &value)?;
        let exact = scope.evaluate(&units("(() => { const d = Object.getOwnPropertyDescriptor(described, 'data'); return d.value === describedValue && d.writable === false && d.enumerable === false && d.configurable === false && !('get' in d) && !('set' in d); })()"))?;
        assert!(scope.boolean(&exact)?);
        assert!(matches!(scope.define_property(&object, PropertyKey::String(&units("data")), PropertyDescriptor::Data { value: &value, writable: true, enumerable: false, configurable: false }), Err(Error::PropertyRejected)));
        scope.define_property(&object, PropertyKey::String(&units("mutable")), PropertyDescriptor::Data { value: &value, writable: true, enumerable: true, configurable: true })?;
        let exact = scope.evaluate(&units("(() => { const d = Object.getOwnPropertyDescriptor(described, 'mutable'); return d.value === describedValue && d.writable && d.enumerable && d.configurable && !('get' in d) && !('set' in d); })()"))?;
        assert!(scope.boolean(&exact)?);
        let get = scope.function(&Callback::new(|scope, arguments| {
            Ok(Completion::Return(scope.get(arguments.receiver(), PropertyKey::String(&units("stored")))?))
        }))?;
        let set = scope.function(&Callback::new(|scope, arguments| {
            scope.set(arguments.receiver(), PropertyKey::String(&units("stored")), arguments.get(0).ok_or(Error::BackendUnavailable)?)?;
            Ok(Completion::Undefined)
        }))?;
        scope.define_property(&object, PropertyKey::String(&units("accessor")), PropertyDescriptor::Accessor { get: Some(&get), set: Some(&set), enumerable: true, configurable: true })?;
        scope.set(&object, PropertyKey::String(&units("accessor")), &value)?;
        let read = scope.get(&object, PropertyKey::String(&units("accessor")))?;
        assert!(scope.strict_equals(&read, &value)?);
        scope.set(&global, PropertyKey::String(&units("descriptorGet")), &get)?;
        scope.set(&global, PropertyKey::String(&units("descriptorSet")), &set)?;
        let exact = scope.evaluate(&units("(() => { const d = Object.getOwnPropertyDescriptor(described, 'accessor'); return d.get === descriptorGet && d.set === descriptorSet && d.enumerable && d.configurable && !('value' in d) && !('writable' in d); })()"))?;
        assert!(scope.boolean(&exact)?);
        let undefined = scope.primitive(Primitive::Undefined)?;
        scope.define_property(&object, PropertyKey::String(&units("empty")), PropertyDescriptor::Accessor { get: None, set: Some(&undefined), enumerable: false, configurable: true })?;
        let exact = scope.evaluate(&units("(() => { const d = Object.getOwnPropertyDescriptor(described, 'empty'); return d.get === undefined && d.set === undefined && !d.enumerable && d.configurable; })()"))?;
        assert!(scope.boolean(&exact)?);
        let callable = scope.evaluate(&units("new Proxy(() => 1, {})"))?;
        scope.define_property(&object, PropertyKey::String(&units("callable")), PropertyDescriptor::Accessor { get: Some(&callable), set: None, enumerable: false, configurable: true })?;
        for source in ["null", "0", "'not callable'", "({get call() {throw 1}})"] {
            let invalid = scope.evaluate(&units(source))?;
            for descriptor in [PropertyDescriptor::Accessor { get: Some(&invalid), set: None, enumerable: true, configurable: true }, PropertyDescriptor::Accessor { get: None, set: Some(&invalid), enumerable: true, configurable: true }] {
                assert!(matches!(scope.define_property(&object, PropertyKey::String(&units("invalid")), descriptor), Err(Error::ExpectedAccessor)));
            }
        }
        let rejects = scope.evaluate(&units("new Proxy({}, {defineProperty() { return false; }})"))?;
        assert!(matches!(scope.define_property(&rejects, PropertyKey::String(&units("x")), PropertyDescriptor::Data { value: &value, writable: true, enumerable: true, configurable: true }), Err(Error::PropertyRejected)));
        let thrown = scope.evaluate(&units("globalThis.descriptorThrown = {}; new Proxy({}, {defineProperty() { throw descriptorThrown; }})"))?;
        let exception = match scope.define_property(&thrown, PropertyKey::String(&units("x")), PropertyDescriptor::Data { value: &value, writable: true, enumerable: true, configurable: true }) { Err(Error::Exception(exception)) => exception, _ => return Err(Error::BackendUnavailable) };
        assert_eq!(exception.phase(), ExceptionPhase::DefineProperty);
        let actual = scope.import(exception.value())?;
        let original = scope.evaluate(&units("descriptorThrown"))?;
        assert!(scope.strict_equals(&actual, &original)?);
        // Reenter these same public primitives from a proxy's define trap.
        let reenter = scope.function(&Callback::new(|scope, _| {
            let object = scope.object(ObjectPrototype::Null)?;
            let value = scope.string(&units("nested"))?;
            scope.define_property(&object, PropertyKey::String(&units("x")), PropertyDescriptor::Data { value: &value, writable: true, enumerable: true, configurable: true })?;
            Ok(Completion::Return(scope.get(&object, PropertyKey::String(&units("x")))?))
        }))?;
        scope.set(&global, PropertyKey::String(&units("reenterDefine")), &reenter)?;
        let proxy = scope.evaluate(&units("globalThis.definitionReentry = ''; new Proxy({}, {defineProperty(target, key, descriptor) { definitionReentry = reenterDefine(); return Reflect.defineProperty(target, key, descriptor); }})"))?;
        scope.define_property(&proxy, PropertyKey::String(&units("x")), PropertyDescriptor::Data { value: &value, writable: true, enumerable: true, configurable: true })?;
        let observed = scope.evaluate(&units("definitionReentry"))?;
        assert_eq!(scope.to_utf16(&observed)?, units("nested"));
        Ok::<_, Error>(())
    })?;
    Ok(())
}

fn new_property_refuses_own_keys_and_preserves_proxy_probe_exceptions(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let object = scope.evaluate(&units(
            "Object.create({chosen: 0}, {ownUndefined: {value: undefined, configurable: true}})",
        ))?;
        let value = scope.primitive(Primitive::Number(7.0))?;
        let descriptor = || PropertyDescriptor::Data {
            value: &value,
            writable: true,
            enumerable: true,
            configurable: true,
        };
        let inherited = units("chosen");
        let own = units("ownUndefined");
        assert!(matches!(
            scope.define_new_property(&object, PropertyKey::String(&own), descriptor()),
            Err(Error::PropertyRejected)
        ));
        let original = scope.get(&object, PropertyKey::String(&own))?;
        assert_eq!(scope.kind(&original)?, ValueKind::Undefined);
        scope.define_new_property(&object, PropertyKey::String(&inherited), descriptor())?;
        let observed = scope.get(&object, PropertyKey::String(&inherited))?;
        assert!(scope.strict_equals(&observed, &value)?);
        assert!(matches!(
            scope.define_new_property(&object, PropertyKey::String(&inherited), descriptor()),
            Err(Error::PropertyRejected)
        ));

        let proxy = scope.evaluate(&units(
            "globalThis.probeFailure = {}; globalThis.definitionCalls = 0;
             new Proxy({}, {
                 getOwnPropertyDescriptor() { throw probeFailure; },
                 defineProperty() { definitionCalls++; return true; }
             })",
        ))?;
        let Err(Error::Exception(exception)) =
            scope.define_new_property(&proxy, PropertyKey::String(&own), descriptor())
        else {
            return Err(Error::BackendUnavailable);
        };
        assert_eq!(exception.phase(), ExceptionPhase::DefineProperty);
        let original = scope.evaluate(&units("probeFailure"))?;
        let thrown = scope.import(exception.value())?;
        assert!(scope.strict_equals(&thrown, &original)?);
        let calls = scope.evaluate(&units("definitionCalls"))?;
        let zero = scope.primitive(Primitive::Number(0.0))?;
        assert!(scope.strict_equals(&calls, &zero)?);
        Ok(())
    })
}

fn objects_prototypes_and_dense_arrays_preserve_identity(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        let default = scope.object(ObjectPrototype::Default)?;
        let null = scope.object(ObjectPrototype::Null)?;
        let custom = scope.object(ObjectPrototype::Object(&null))?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units("defaultObject")), &default)?;
        scope.set(&global, PropertyKey::String(&units("nullObject")), &null)?;
        scope.set(&global, PropertyKey::String(&units("customObject")), &custom)?;
        let correct = scope.evaluate(&units("Object.getPrototypeOf(defaultObject) === Object.prototype && Object.getPrototypeOf(nullObject) === null && Object.getPrototypeOf(customObject) === nullObject"))?;
        assert!(scope.boolean(&correct)?);
        scope.set_prototype(&custom, ObjectPrototype::Default)?;
        scope.set_prototype(&default, ObjectPrototype::Null)?;
        let correct = scope.evaluate(&units("Object.getPrototypeOf(defaultObject) === null && Object.getPrototypeOf(customObject) === Object.prototype"))?;
        assert!(scope.boolean(&correct)?);
        scope.set_prototype(&custom, ObjectPrototype::Object(&default))?;
        assert!(matches!(scope.set_prototype(&default, ObjectPrototype::Object(&custom)), Err(Error::PropertyRejected)));
        let primitive = scope.primitive(Primitive::Number(1.0))?;
        assert!(matches!(scope.object(ObjectPrototype::Object(&primitive)), Err(Error::ExpectedObject)));
        assert!(matches!(scope.set_prototype(&primitive, ObjectPrototype::Null), Err(Error::ExpectedObject)));
        assert!(matches!(scope.set_prototype(&custom, ObjectPrototype::Object(&primitive)), Err(Error::ExpectedObject)));
        let frozen = scope.evaluate(&units("Object.preventExtensions({})"))?;
        assert!(matches!(scope.set_prototype(&frozen, ObjectPrototype::Null), Err(Error::PropertyRejected)));
        let rejects = scope.evaluate(&units("new Proxy({}, {setPrototypeOf() {return false}})"))?;
        assert!(matches!(scope.set_prototype(&rejects, ObjectPrototype::Null), Err(Error::PropertyRejected)));
        let throwing = scope.evaluate(&units("globalThis.prototypeThrown = {}; new Proxy({}, {setPrototypeOf() {throw prototypeThrown}})"))?;
        let exception = match scope.set_prototype(&throwing, ObjectPrototype::Null) { Err(Error::Exception(exception)) => exception, _ => return Err(Error::BackendUnavailable) };
        assert_eq!(exception.phase(), ExceptionPhase::Prototype);
        let thrown = scope.import(exception.value())?;
        let original = scope.evaluate(&units("prototypeThrown"))?;
        assert!(scope.strict_equals(&thrown, &original)?);
        let values = [scope.object(ObjectPrototype::Null)?, scope.primitive(Primitive::Undefined)?, scope.string(&[0xd800])?];
        let array = scope.array(&values)?;
        let first = scope.get(&array, PropertyKey::String(&units("0")))?;
        assert!(scope.strict_equals(&first, &values[0])?);
        scope.set(&global, PropertyKey::String(&units("denseArray")), &array)?;
        let exact = scope.evaluate(&units("Array.isArray(denseArray) && denseArray.length === 3 && [0, 1, 2].every(index => Object.hasOwn(denseArray, index)) && denseArray[1] === undefined && denseArray[2].charCodeAt(0) === 0xd800 && !Object.hasOwn(denseArray, 3)"))?;
        assert!(scope.boolean(&exact)?);
        let empty = scope.array(&[])?;
        let length = scope.get(&empty, PropertyKey::String(&units("length")))?;
        assert_eq!(scope.to_utf16(&length)?, units("0"));
        Ok::<_, Error>(())
    })?;
    Ok(())
}

fn captured_intrinsics_survive_guest_replacement_and_realm_lease_release(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let a = execution.create_realm()?;
    let b = execution.create_realm()?;
    let drops = Rc::new(Cell::new(0));
    let receipt = DropReceipt(drops.clone());
    let callback = Callback::new(move |scope, _| {
        let _capture = &receipt;
        let value = scope.primitive(Primitive::Number(7.0))?;
        let array = scope.array(&[value])?;
        let iterator_method = scope.intrinsic(Intrinsic::ArrayValues)?;
        let iterator = scope.call(&iterator_method, &array, &[])?;
        let next = scope.get(&iterator, PropertyKey::String(&units("next")))?;
        let result = scope.call(&next, &iterator, &[])?;
        let value = scope.get(&result, PropertyKey::String(&units("value")))?;
        let object = scope.object(ObjectPrototype::Default)?;
        scope.set_prototype(&object, ObjectPrototype::Null)?;
        scope.set(&object, PropertyKey::String(&units("value")), &value)?;
        Ok(Completion::Return(
            scope.get(&object, PropertyKey::String(&units("value")))?,
        ))
    });
    let root = execution.in_realm(&a, |scope| {
        let original = scope.intrinsic(Intrinsic::ArrayValues)?;
        let actual = scope.evaluate(&units("Array.prototype.values"))?;
        assert!(scope.strict_equals(&original, &actual)?);
        let global = scope.global()?;
        let function = scope.function(&callback)?;
        scope.set(&global, PropertyKey::String(&units("cycle")), &function)?;
        Ok::<_, Error>(scope.retain(&function))
    })?;
    execution.in_realm(&b, |scope| {
        let global = scope.global()?;
        let function = scope.import(&root)?;
        scope.set(
            &global,
            PropertyKey::String(&units("fromAIntrinsic")),
            &function,
        )
    })?;
    evaluate(
        &mut execution,
        &a,
        "Reflect.set = Reflect.setPrototypeOf = Array.prototype.values = () => { throw 'replaced intrinsic'; }; globalThis.Array = null;",
    )?;
    drop(root);
    drop(callback);
    drop(a);
    execution.memory_pressure();
    assert_eq!(
        evaluate(&mut execution, &b, "fromAIntrinsic()")?,
        units("7")
    );
    assert_eq!(drops.get(), 0);
    evaluate(&mut execution, &b, "fromAIntrinsic = null")?;
    evaluate(&mut execution, &b, "gc()")?;
    execution.memory_pressure();
    assert_eq!(drops.get(), 1);
    Ok(())
}

fn managed_holder_preserves_object_and_creation_realm_until_release(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let a = execution.create_realm()?;
    let b = execution.create_realm()?;
    let drops = Rc::new(Cell::new(0));
    let receipt = DropReceipt(drops.clone());
    let callback = Callback::new(move |scope, _| {
        let _receipt = &receipt;
        Ok(Completion::Return(scope.evaluate(&units("realmMarker"))?))
    });
    let holder = execution.in_realm(&a, |scope| {
        scope.evaluate(&units("globalThis.realmMarker = 41"))?;
        let holder = scope.managed_object()?;
        let object = scope.open_managed_object(&holder)?;
        let marker = scope.function(&callback)?;
        scope.set(&object, PropertyKey::String(&units("marker")), &marker)?;
        Ok(holder)
    })?;
    drop(callback);
    drop(a);
    evaluate(&mut execution, &b, "gc()")?;
    execution.memory_pressure();
    assert_eq!(drops.get(), 0);
    execution.in_realm(&b, |scope| {
        let object = scope.open_managed_object(&holder)?;
        let again = scope.open_managed_object(&holder)?;
        assert!(scope.strict_equals(&object, &again)?);
        let marker = scope.get(&object, PropertyKey::String(&units("marker")))?;
        let result = scope.call(&marker, &object, &[])?;
        let expected = scope.primitive(Primitive::Number(41.0))?;
        assert!(scope.strict_equals(&result, &expected)?);
        Ok(())
    })?;
    drop(execution);
    process.release_managed_object(holder)?;
    let mut execution = process.execution();
    evaluate(&mut execution, &b, "gc()")?;
    execution.memory_pressure();
    assert_eq!(drops.get(), 1);
    Ok(())
}

fn new_object_paths_preserve_the_first_host_failure() -> Result<(), Error> {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let checks = Rc::new(Cell::new(0));
    install(
        &mut execution,
        &realm,
        "failNewPaths",
        &Callback::new(|_, _| Err(Error::IntegerOutOfRange)),
    )?;
    let observed = checks.clone();
    install(
        &mut execution,
        &realm,
        "testNewPaths",
        &Callback::new(move |scope, _| {
            let object = scope.object(ObjectPrototype::Null)?;
            let value = scope.primitive(Primitive::Undefined)?;
            let values = [scope.primitive(Primitive::Null)?];
            let _failure = scope.evaluate(&units("try { failNewPaths() } catch (_) {}"));
            // Record refusal results outside guest code. Assertions inside a failed
            // callback could be masked by the already sticky first host failure.
            let mut count = 0;
            count += usize::from(matches!(
                scope.get(&object, PropertyKey::Value(&value)),
                Err(Error::IntegerOutOfRange)
            ));
            count += usize::from(matches!(
                scope.set(&object, PropertyKey::Value(&value), &value),
                Err(Error::IntegerOutOfRange)
            ));
            count += usize::from(matches!(
                scope.define_property(
                    &object,
                    PropertyKey::Value(&value),
                    PropertyDescriptor::Data {
                        value: &value,
                        writable: true,
                        enumerable: true,
                        configurable: true
                    }
                ),
                Err(Error::IntegerOutOfRange)
            ));
            count += usize::from(matches!(
                scope.object(ObjectPrototype::Default),
                Err(Error::IntegerOutOfRange)
            ));
            count += usize::from(matches!(
                scope.set_prototype(&object, ObjectPrototype::Default),
                Err(Error::IntegerOutOfRange)
            ));
            count += usize::from(matches!(
                scope.array(&values),
                Err(Error::IntegerOutOfRange)
            ));
            count += usize::from(matches!(
                scope.well_known_symbol(WellKnownSymbol::Iterator),
                Err(Error::IntegerOutOfRange)
            ));
            count += usize::from(matches!(
                scope.intrinsic(Intrinsic::ArrayValues),
                Err(Error::IntegerOutOfRange)
            ));
            observed.set(count);
            Ok(Completion::Undefined)
        }),
    )?;
    assert!(matches!(
        evaluate(&mut execution, &realm, "testNewPaths()"),
        Err(Error::IntegerOutOfRange)
    ));
    assert_eq!(checks.get(), 8);
    Ok(())
}

fn install_constructor(
    execution: &mut Execution<'_>,
    realm: &Realm,
    name: &str,
    length: u32,
    callback: &ConstructorCallback,
) -> Result<(), Error> {
    execution.in_realm(realm, |scope| {
        let function = scope.constructor(
            callback,
            FunctionMetadata {
                name: &units(name),
                length,
            },
        )?;
        let global = scope.global()?;
        scope.set(&global, PropertyKey::String(&units(name)), &function)
    })
}

fn fixture_constructor(
    effects: Rc<Cell<usize>>,
    fallback_calls: Rc<Cell<usize>>,
    receipt: DropReceipt,
) -> ConstructorCallback {
    ConstructorCallback::new(
        move |scope| {
            fallback_calls.set(fallback_calls.get() + 1);
            let global = scope.global()?;
            scope.get(&global, PropertyKey::String(&units("fixturePrototype")))
        },
        move |scope, arguments| {
            let _capture = &receipt;
            let argument = match arguments.get(0) {
                Some(value) => scope.integer(value, IntegerKind::Long, IntegerMode::Wrap)?,
                None => IntegerValue::Long(0),
            };
            // Actual typed adapters convert their arguments BEFORE this
            // consuming prototype-selection transition and before native action.
            let invocation = arguments.select_prototype(scope)?;
            scope.check_active()?;
            effects.set(effects.get() + 1);
            let object = scope.object(ObjectPrototype::Object(invocation.prototype()))?;
            let global = scope.global()?;
            let realm = scope.get(&global, PropertyKey::String(&units("realmLabel")))?;
            scope.set(&object, PropertyKey::String(&units("nativeRealm")), &realm)?;
            scope.set(
                &object,
                PropertyKey::String(&units("newTarget")),
                invocation.new_target(),
            )?;
            let IntegerValue::Long(argument) = argument else {
                return Err(Error::BackendUnavailable);
            };
            let value = scope.primitive(Primitive::Number(f64::from(argument)))?;
            scope.set(&object, PropertyKey::String(&units("argument")), &value)?;
            Ok(ConstructorCompletion::Return(object))
        },
    )
}

fn declared_constructors_preserve_calls_metadata_and_installed_descriptors(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    evaluate(
        &mut execution,
        &realm,
        "globalThis.realmLabel = 'A'; globalThis.fixturePrototype = {}",
    )?;
    let effects = Rc::new(Cell::new(0));
    let fallback = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let callback = fixture_constructor(effects.clone(), fallback, DropReceipt(drops));
    install_constructor(&mut execution, &realm, "Declared", 2, &callback)?;
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "Declared.name === 'Declared' && Declared.length === 2 && Declared.prototype.constructor === Declared"
        )?,
        units("true")
    );
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "['name', 'length'].every(key => { const d = Object.getOwnPropertyDescriptor(Declared, key); return !d.writable && !d.enumerable && d.configurable; })"
        )?,
        units("true")
    );
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "try { Declared(); false } catch (error) { error instanceof TypeError }"
        )?,
        units("true")
    );
    assert_eq!(effects.get(), 0);
    let entries = Rc::new(Cell::new(0));
    let observed_entries = entries.clone();
    let guard = ConstructorCallback::new(
        |scope| scope.object(ObjectPrototype::Default),
        move |_, _| {
            observed_entries.set(observed_entries.get() + 1);
            Err(Error::BackendUnavailable)
        },
    );
    install_constructor(&mut execution, &realm, "Guarded", 0, &guard)?;
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "try { Guarded.call({}, 1); false } catch (error) { error instanceof TypeError }"
        )?,
        units("true")
    );
    assert_eq!(
        entries.get(),
        0,
        "plain calls must not enter the handler at all"
    );
    execution.in_realm(&realm, |scope| {
        let global = scope.global()?;
        let function = scope.get(&global, PropertyKey::String(&units("Declared")))?;
        let prototype = scope.object(ObjectPrototype::Default)?;
        scope.define_property(
            &prototype,
            PropertyKey::String(&units("constructor")),
            PropertyDescriptor::Data {
                value: &function,
                writable: true,
                enumerable: false,
                configurable: true,
            },
        )?;
        scope.define_property(
            &function,
            PropertyKey::String(&units("prototype")),
            PropertyDescriptor::Data {
                value: &prototype,
                writable: false,
                enumerable: false,
                configurable: false,
            },
        )?;
        let units = [0xd800, 0, 0xdc00];
        let exact = scope.constructor(
            &callback,
            FunctionMetadata {
                name: &units,
                length: 3,
            },
        )?;
        let name = scope.get(&exact, PropertyKey::String(&[110, 97, 109, 101]))?;
        assert_eq!(scope.to_utf16(&name)?, units);
        Ok::<_, Error>(())
    })?;
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "(() => { const d = Object.getOwnPropertyDescriptor(Declared, 'prototype'); const value = new Declared(7); return !d.writable && !d.enumerable && !d.configurable && Object.getPrototypeOf(value) === Declared.prototype && value.newTarget === Declared && value.nativeRealm === 'A' && value.argument === 7; })()"
        )?,
        units("true")
    );
    assert_eq!(effects.get(), 1);
    Ok(())
}

fn declared_constructor_length_is_exact_u32(process: &mut Process) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    let effects = Rc::new(Cell::new(0));
    let observed_effects = effects.clone();
    let callback = ConstructorCallback::new(
        |scope| scope.object(ObjectPrototype::Default),
        move |scope, arguments| {
            let invocation = arguments.select_prototype(scope)?;
            scope.check_active()?;
            observed_effects.set(observed_effects.get() + 1);
            Ok(ConstructorCompletion::Return(
                scope.object(ObjectPrototype::Object(invocation.prototype()))?,
            ))
        },
    );
    let mut failures = Vec::new();
    for length in [65_535_u32, 65_536, 1_000_000, u32::MAX] {
        let before = effects.get();
        let result = execution.in_realm(&realm, |scope| {
            let function = match scope.constructor(&callback, FunctionMetadata { name: &units("LengthBoundary"), length }) {
                Ok(function) => function,
                Err(error) => return Ok::<_, Error>(format!("creation refused: {error}")),
            };
            let global = scope.global()?;
            scope.set(&global, PropertyKey::String(&units("LengthBoundary")), &function)?;
            // The public own descriptor and bind read observable metadata;
            // construction still executes the real typed native callback.
            let value = scope.evaluate(&units("(() => { const descriptor = Object.getOwnPropertyDescriptor(LengthBoundary, 'length'); const bound = LengthBoundary.bind(null, 1); let refused = false; try { LengthBoundary(); } catch (error) { refused = error instanceof TypeError; } const constructed = new LengthBoundary() instanceof LengthBoundary && new bound() instanceof LengthBoundary; return [descriptor.value, [descriptor.writable, descriptor.enumerable, descriptor.configurable].join(','), bound.length, refused, constructed].join('|'); })()"))?;
            Ok(String::from_utf16_lossy(&scope.to_utf16(&value)?))
        })?;
        let expected = format!("{length}|false,false,true|{}|true|true", length - 1);
        eprintln!("constructor length {length}: {result}");
        if result != expected {
            failures.push(format!(
                "length {length}: expected {expected}, observed {result}"
            ));
        }
        if effects.get() - before != 2 {
            failures.push(format!(
                "length {length}: expected two native constructions, observed {}",
                effects.get() - before
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn declared_constructor_prototype_get_follows_conversions_and_preserves_abrupt(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    evaluate(
        &mut execution,
        &realm,
        "globalThis.realmLabel = 'A'; globalThis.fixturePrototype = {}; globalThis.order = []; globalThis.prototypeReads = 0; globalThis.originalAbrupt = {}; globalThis.Other = new Proxy(function Other(){}, {get(target, key, receiver) { if (key === 'prototype') { order.push('prototype'); prototypeReads++; throw originalAbrupt; } return Reflect.get(target, key, receiver); }})",
    )?;
    let effects = Rc::new(Cell::new(0));
    let callback = fixture_constructor(
        effects.clone(),
        Rc::new(Cell::new(0)),
        DropReceipt(Rc::new(Cell::new(0))),
    );
    install_constructor(&mut execution, &realm, "Declared", 0, &callback)?;
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "try { Reflect.construct(Declared, [{valueOf() {order.push('convert'); return 1}}], Other); false } catch (error) { error === originalAbrupt && order.join(',') === 'convert,prototype' && prototypeReads === 1 }"
        )?,
        units("true")
    );
    assert_eq!(effects.get(), 0);
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "order = []; try { Reflect.construct(Declared, [{valueOf() {order.push('convert'); throw originalAbrupt}}], Other); false } catch (error) { error === originalAbrupt && order.join(',') === 'convert' && prototypeReads === 1 }"
        )?,
        units("true")
    );
    assert_eq!(effects.get(), 0);
    let throws = ConstructorCallback::new(
        |scope| scope.object(ObjectPrototype::Default),
        |_scope, arguments| {
            let (mut values, _) = arguments.into_parts();
            Ok(ConstructorCompletion::Throw(
                values.pop().ok_or(Error::BackendUnavailable)?,
            ))
        },
    );
    install_constructor(&mut execution, &realm, "ExplicitThrow", 1, &throws)?;
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "try { new ExplicitThrow(originalAbrupt); false } catch (error) { error === originalAbrupt }"
        )?,
        units("true")
    );
    Ok(())
}

fn declared_constructor_cross_realm_fallback_is_once_and_guest_intrinsic_independent(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let a = execution.create_realm()?;
    let b = execution.create_realm()?;
    let driver = execution.create_realm()?;
    evaluate(
        &mut execution,
        &a,
        "globalThis.realmLabel = 'A'; globalThis.fixturePrototype = {fallbackRealm:'A'}",
    )?;
    evaluate(
        &mut execution,
        &b,
        "globalThis.realmLabel = 'B'; globalThis.fixturePrototype = {fallbackRealm:'B'}",
    )?;
    let effects = Rc::new(Cell::new(0));
    let fallback = Rc::new(Cell::new(0));
    let drops = Rc::new(Cell::new(0));
    let callback = fixture_constructor(
        effects.clone(),
        fallback.clone(),
        DropReceipt(drops.clone()),
    );
    install_constructor(&mut execution, &a, "Declared", 1, &callback)?;
    let (constructor, a_type_error) = execution.in_realm(&a, |scope| {
        let global = scope.global()?;
        let constructor = scope.get(&global, PropertyKey::String(&units("Declared")))?;
        let error = scope.get(&global, PropertyKey::String(&units("TypeError")))?;
        Ok::<_, Error>((scope.retain(&constructor), scope.retain(&error)))
    })?;
    execution.in_realm(&b, |scope| {
        let global = scope.global()?;
        let constructor = scope.import(&constructor)?;
        let error = scope.import(&a_type_error)?;
        scope.set(&global, PropertyKey::String(&units("fromA")), &constructor)?;
        scope.set(&global, PropertyKey::String(&units("TypeErrorA")), &error)
    })?;
    evaluate(
        &mut execution,
        &b,
        "globalThis.prototypeReads = 0; globalThis.extraDescriptorReads = 0; globalThis.extraPrototypeReads = 0; globalThis.metadataReads = 0; globalThis.argumentConversions = 0; globalThis.events = []; const originalConstruct = Reflect.construct; const reflectGet = Reflect.get; const bind = Function.prototype.bind; const freeze = Object.freeze; const define = Object.defineProperty; const getPrototype = Object.getPrototypeOf; globalThis.objectPrototype = {selected:'B-object'}; globalThis.objectTarget = new Proxy(function Other(){}, {get(target,key,receiver) { if (key === 'prototype') {prototypeReads++; events.push('prototype'); return objectPrototype;} return reflectGet(target,key,receiver); }}); const bound = bind.call(function BoundBase(){}, null); for (const key of ['name', 'length']) { define(bound, key, {get() {metadataReads++; throw 'metadata accessor must not run';}, configurable:true}); } define(bound, 'prototype', {get() {prototypeReads++; events.push('prototype'); freeze(bound); return 0;}, configurable:true}); globalThis.frozenTarget = new Proxy(bound, {get: reflectGet, getOwnPropertyDescriptor() {extraDescriptorReads++; throw 'unexpected descriptor trap';}, getPrototypeOf() {extraPrototypeReads++; throw 'unexpected prototype trap';}}); const inner = new Proxy(function DeepBase(){}, {}); globalThis.boundProxyTarget = bind.call(inner, null); define(boundProxyTarget, 'prototype', {value: 0, writable:false, configurable:false}); globalThis.launch = mode => { events.length = 0; const target = mode === 'object' ? objectTarget : mode === 'frozen' ? frozenTarget : boundProxyTarget; const result = originalConstruct(fromA, [{valueOf() {argumentConversions++; events.push('convert'); return 9;}}], target); return result.nativeRealm === 'A' && result.newTarget === target && result.argument === 9 && getPrototype(result) === (mode === 'object' ? objectPrototype : fixturePrototype) && extraDescriptorReads === 0 && extraPrototypeReads === 0 && metadataReads === 0 && (mode === 'bound' || events.join(',') === 'convert,prototype'); }; globalThis.plainCall = () => { try { fromA() } catch (error) { return error instanceof TypeErrorA && !(error instanceof TypeError); } }; globalThis.poison = () => { for (const key of ['get', 'getOwnPropertyDescriptor', 'getPrototypeOf', 'construct', 'apply', 'value']) { define(Object.prototype, key, {get() { throw 'inherited handler/descriptor pollution'; }, configurable:true}); } Reflect.construct = Function.prototype.bind = () => {throw 'replaced intrinsic';}; globalThis.Object = null; }; 0",
    )?;
    // Create a Proxy and then its bound shell in A around a function from B.
    // The bound object's allocation context A is NOT its true function realm B.
    let b_target = execution.in_realm(&b, |scope| {
        let value = scope.evaluate(&units("function CrossRealmTarget(){}; CrossRealmTarget"))?;
        Ok::<_, Error>(scope.retain(&value))
    })?;
    execution.in_realm(&a, |scope| {
        let global = scope.global()?;
        let target = scope.import(&b_target)?;
        scope.set(&global, PropertyKey::String(&units("targetFromB")), &target)
    })?;
    let cross_bound = execution.in_realm(&a, |scope| {
        let value = scope.evaluate(&units("globalThis.crossBound = Function.prototype.bind.call(new Proxy(targetFromB, {}), null); Object.defineProperty(crossBound, 'prototype', {value:0, writable:false, configurable:false}); Object.defineProperty(crossBound, 'name', {value:{toString() {throw 'name data must not coerce';}}, configurable:false}); Object.defineProperty(crossBound, 'length', {value:{valueOf() {throw 'length data must not coerce';}}, configurable:false}); crossBound"))?;
        Ok::<_, Error>(scope.retain(&value))
    })?;
    execution.in_realm(&b, |scope| {
        let global = scope.global()?;
        let value = scope.import(&cross_bound)?;
        scope.set(
            &global,
            PropertyKey::String(&units("boundProxyTarget")),
            &value,
        )
    })?;
    drop(b_target);
    drop(cross_bound);
    let (launch, plain_call, poison) = execution.in_realm(&b, |scope| {
        let global = scope.global()?;
        let launch = scope.get(&global, PropertyKey::String(&units("launch")))?;
        let plain = scope.get(&global, PropertyKey::String(&units("plainCall")))?;
        let poison = scope.get(&global, PropertyKey::String(&units("poison")))?;
        Ok::<_, Error>((
            scope.retain(&launch),
            scope.retain(&plain),
            scope.retain(&poison),
        ))
    })?;
    execution.in_realm(&driver, |scope| {
        let global = scope.global()?;
        for (name, root) in [
            ("launchB", &launch),
            ("plainB", &plain_call),
            ("poisonB", &poison),
        ] {
            let value = scope.import(root)?;
            scope.set(&global, PropertyKey::String(&units(name)), &value)?;
        }
        Ok::<_, Error>(())
    })?;
    // Poison A's inherited trap/descriptor fields too. The core's private
    // handler objects must be null-prototype and descriptor.value must be own.
    evaluate(
        &mut execution,
        &a,
        "for (const key of ['get', 'getOwnPropertyDescriptor', 'getPrototypeOf', 'construct', 'apply', 'value']) { Object.defineProperty(Object.prototype, key, {get() {throw 'A inherited trap/descriptor pollution';}, configurable:true}); } Reflect.construct = Function.prototype.bind = () => {throw 'A replaced intrinsic';}; globalThis.Object = null;",
    )?;
    assert_eq!(
        evaluate(&mut execution, &driver, "poisonB(); plainB()")?,
        units("true")
    );
    assert_eq!(effects.get(), 0);
    drop(constructor);
    drop(a_type_error);
    drop(launch);
    drop(plain_call);
    drop(poison);
    drop(callback);
    drop(a);
    drop(b);
    execution.memory_pressure();
    assert_eq!(
        evaluate(&mut execution, &driver, "launchB('object')")?,
        units("true")
    );
    assert_eq!(fallback.get(), 0);
    assert_eq!(
        evaluate(&mut execution, &driver, "launchB('frozen')")?,
        units("true")
    );
    assert_eq!(fallback.get(), 1);
    assert_eq!(
        evaluate(&mut execution, &driver, "launchB('bound')")?,
        units("true")
    );
    assert_eq!(fallback.get(), 2);
    assert_eq!(effects.get(), 3);
    assert_eq!(drops.get(), 0);
    evaluate(&mut execution, &driver, "launchB = plainB = poisonB = null")?;
    evaluate(&mut execution, &driver, "gc()")?;
    execution.memory_pressure();
    assert_eq!(drops.get(), 1);
    Ok(())
}

fn declared_constructor_revocation_stops_before_fallback_and_native_action(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let realm = execution.create_realm()?;
    evaluate(
        &mut execution,
        &realm,
        "globalThis.realmLabel = 'A'; globalThis.fixturePrototype = {}; const bind = Function.prototype.bind; globalThis.duringGet = Proxy.revocable(function DuringGet(){}, {get(target,key,receiver) {if (key === 'prototype') {duringGet.revoke(); return 0;} return Reflect.get(target,key,receiver);}}); const inner = Proxy.revocable(function Inner(){}, {}); globalThis.revokedBound = bind.call(inner.proxy, null); inner.revoke(); 0",
    )?;
    let effects = Rc::new(Cell::new(0));
    let fallback = Rc::new(Cell::new(0));
    let callback = fixture_constructor(
        effects.clone(),
        fallback.clone(),
        DropReceipt(Rc::new(Cell::new(0))),
    );
    install_constructor(&mut execution, &realm, "Declared", 0, &callback)?;
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "[duringGet.proxy, revokedBound].every(target => {try {Reflect.construct(Declared, [], target); return false;} catch (error) {return error instanceof TypeError;}})"
        )?,
        units("true")
    );
    assert_eq!(effects.get(), 0);
    assert_eq!(fallback.get(), 0);
    evaluate(
        &mut execution,
        &realm,
        "globalThis.selectedBeforeRevoke = {}; globalThis.objectDuringGet = Proxy.revocable(function ObjectDuringGet(){}, {get(target,key,receiver) {if (key === 'prototype') {objectDuringGet.revoke(); return selectedBeforeRevoke;} return Reflect.get(target,key,receiver);}})",
    )?;
    assert_eq!(
        evaluate(
            &mut execution,
            &realm,
            "Object.getPrototypeOf(Reflect.construct(Declared, [], objectDuringGet.proxy)) === selectedBeforeRevoke"
        )?,
        units("true")
    );
    assert_eq!(
        effects.get(),
        1,
        "an object prototype does not require GetFunctionRealm after revocation"
    );
    assert_eq!(fallback.get(), 0);
    Ok(())
}

fn declared_constructor_host_refusals_are_terminal() -> Result<(), Error> {
    for invalid_return in [false, true] {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let mut execution = process.execution();
        let realm = execution.create_realm()?;
        let effects = Rc::new(Cell::new(0));
        let observed = effects.clone();
        let callback = ConstructorCallback::new(
            move |scope| {
                if invalid_return {
                    scope.object(ObjectPrototype::Default)
                } else {
                    Err(Error::IntegerOutOfRange)
                }
            },
            move |scope, arguments| {
                let thrown = scope.string(&units("later throw"))?;
                let result = arguments.select_prototype(scope);
                if scope.check_active().is_ok() {
                    observed.set(observed.get() + 1);
                }
                if invalid_return {
                    result?;
                    Ok(ConstructorCompletion::Return(
                        scope.primitive(Primitive::Number(0.0))?,
                    ))
                } else {
                    // The first host failure overrides this later guest throw.
                    Ok(ConstructorCompletion::Throw(thrown))
                }
            },
        );
        install_constructor(&mut execution, &realm, "Declared", 0, &callback)?;
        let error = evaluate(
            &mut execution,
            &realm,
            "const target = (function(){}).bind(null); try { Reflect.construct(Declared, [], target) } catch (_) {} 'must not succeed'",
        );
        if invalid_return {
            assert!(matches!(error, Err(Error::ExpectedObject)));
            assert_eq!(effects.get(), 1);
            assert!(matches!(
                execution.create_realm(),
                Err(Error::ExpectedObject)
            ));
        } else {
            assert!(matches!(error, Err(Error::IntegerOutOfRange)));
            assert_eq!(effects.get(), 0);
            assert!(matches!(
                execution.create_realm(),
                Err(Error::IntegerOutOfRange)
            ));
        }
    }
    let drops = Rc::new(Cell::new(0));
    {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let mut execution = process.execution();
        let realm = execution.create_realm()?;
        evaluate(
            &mut execution,
            &realm,
            "globalThis.realmLabel = 'A'; globalThis.fixturePrototype = {}",
        )?;
        let callback = fixture_constructor(
            Rc::new(Cell::new(0)),
            Rc::new(Cell::new(0)),
            DropReceipt(drops.clone()),
        );
        install_constructor(&mut execution, &realm, "Declared", 0, &callback)?;
        drop(callback);
        evaluate(
            &mut execution,
            &realm,
            "Promise.resolve().then(() => new Declared())",
        )?;
        assert_eq!(drops.get(), 0);
    }
    assert_eq!(drops.get(), 1);
    Ok(())
}

fn main() -> Result<(), Error> {
    v8::V8::set_flags_from_string("--expose-gc");
    let mut process = Process::bootstrap(EngineKind::V8)?;
    if std::env::args().nth(1).as_deref() == Some("dom-exception-cross-realm") {
        let drops =
            dom_exception::profile_survives_guest_global_replacement_and_cross_realm_construction(
                &mut process,
            )?;
        drop(process);
        assert_eq!(drops.get(), 1, "the retired profile retained its backing");
        return Ok(());
    }
    if std::env::args().nth(1).as_deref() == Some("--constructor-length") {
        return declared_constructor_length_is_exact_u32(&mut process);
    }
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
    property_keys_are_exact_and_never_coerce(&mut process)?;
    descriptors_preserve_attributes_reentry_and_original_throws(&mut process)?;
    new_property_refuses_own_keys_and_preserves_proxy_probe_exceptions(&mut process)?;
    objects_prototypes_and_dense_arrays_preserve_identity(&mut process)?;
    captured_intrinsics_survive_guest_replacement_and_realm_lease_release(&mut process)?;
    managed_holder_preserves_object_and_creation_realm_until_release(&mut process)?;
    declared_constructors_preserve_calls_metadata_and_installed_descriptors(&mut process)?;
    declared_constructor_length_is_exact_u32(&mut process)?;
    declared_constructor_prototype_get_follows_conversions_and_preserves_abrupt(&mut process)?;
    declared_constructor_cross_realm_fallback_is_once_and_guest_intrinsic_independent(
        &mut process,
    )?;
    declared_constructor_revocation_stops_before_fallback_and_native_action(&mut process)?;
    let dom_exception_drops =
        dom_exception::profile_survives_guest_global_replacement_and_cross_realm_construction(
            &mut process,
        )?;

    scalar_construction_preserves_javascript_values(&mut process)?;
    value_classification_uses_language_types_without_guest_hooks(&mut process)?;
    callable_classification_includes_classes_and_revoked_callable_proxies(&mut process)?;
    scalar_and_classification_operations_work_in_native_callbacks(&mut process)?;
    host::observed_realm_is_the_callback_creation_realm(&mut process)?;

    let (old_realm, old_value) = {
        let mut execution = process.execution();
        let realm = execution.create_realm()?;
        let value = execution.in_realm(&realm, |scope| {
            let value = scope.evaluate(&units("Promise.resolve().then(() => { throw new Error('must not run at teardown') }); ({alive: true})"))?;
            Ok::<_, Error>(scope.retain(&value))
        })?;
        (realm, value)
    };
    drop(process);
    assert_eq!(
        dom_exception_drops.get(),
        1,
        "the retired profile retained its backing"
    );
    let mut next = Process::bootstrap(EngineKind::V8)?;
    let mut execution = next.execution();
    assert!(matches!(
        execution.in_realm(&old_realm, |_| Ok(())),
        Err(Error::ForeignOwner)
    ));
    let realm = execution.create_realm()?;
    execution.in_realm(&realm, |scope| {
        assert!(matches!(scope.import(&old_value), Err(Error::ForeignOwner)));
        Ok(())
    })?;
    // Forgetting a temporary borrow cannot lose ownership of the isolate.
    std::mem::forget(execution);
    let mut execution = next.execution();
    assert_eq!(evaluate(&mut execution, &realm, "42")?, units("42"));
    drop(execution);
    drop(next);
    host_refusals_are_sticky_and_prevent_native_effects()?;
    new_object_paths_preserve_the_first_host_failure()?;
    declared_constructor_host_refusals_are_terminal()?;
    scalar_operations_preserve_the_first_host_failure()?;
    host::refusal_preserves_actual_native_error_and_expiry()?;
    host::prepared_slot_cannot_replace_an_earlier_failure()?;
    host::direct_failure_returns_validate_owner()?;
    host::rejected_failure_drop_stays_in_callback_guard()?;
    println!(
        "execution ownership, realms, roots, UTF-16, exceptions, properties, calls, jobs and native callbacks passed"
    );
    Ok(())
}
