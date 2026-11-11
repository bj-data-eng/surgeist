use super::{evaluate, units};
use surgeist_bindings_js::dom_exception::{self, DomExceptionLimits, DomExceptionName};
use surgeist_bindings_js::{Callback, Completion, Error, Process, PropertyKey};

pub(super) fn profile_survives_guest_global_replacement_and_cross_realm_construction(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let a = execution.create_realm()?;
    let b = execution.create_realm()?;
    let driver = execution.create_realm()?;
    let nonextensible = execution.create_realm()?;
    execution.in_realm(&nonextensible, |scope| {
        // The absent own-key probe succeeds. V8 then refuses the actual
        // definition because its context global is nonextensible.
        scope.evaluate(&units("Object.preventExtensions(globalThis)"))?;
        assert!(matches!(
            dom_exception::install(scope, DomExceptionLimits::default(), ()),
            Err(Error::PropertyRejected)
        ));
        assert!(matches!(
            scope.dom_exception(DomExceptionName::SyntaxError, &[]),
            Err(Error::DomExceptionUnavailable)
        ));
        let key = scope.evaluate(&units("typeof globalThis.DOMException"))?;
        assert_eq!(scope.to_utf16(&key)?, units("undefined"));
        Ok::<_, Error>(())
    })?;
    drop(nonextensible);
    let (constructor, prototype_a, native) = execution.in_realm(&a, |scope| {
        let limits = DomExceptionLimits::default();
        assert!(matches!(
            scope.dom_exception(DomExceptionName::SyntaxError, &[]),
            Err(Error::DomExceptionUnavailable)
        ));
        let script = "globalThis.DOMException = undefined";
        scope.evaluate(&units(script))?;
        assert!(matches!(
            dom_exception::install(scope, limits, ()),
            Err(Error::PropertyRejected)
        ));
        assert!(matches!(
            scope.dom_exception(DomExceptionName::SyntaxError, &[]),
            Err(Error::DomExceptionUnavailable)
        ));
        scope.evaluate(&units("delete globalThis.DOMException"))?;
        dom_exception::install(scope, limits, ())?;
        let constructor = scope.evaluate(&units("DOMException"))?;
        let prototype = scope.evaluate(&units("DOMException.prototype"))?;
        scope.evaluate(&units(
            "delete globalThis.DOMException; Object.defineProperty(globalThis, 'Error', {
                get() { throw 'A Error global lookup'; }
            });",
        ))?;
        assert!(matches!(
            dom_exception::install(scope, limits, ()),
            Err(Error::PropertyRejected)
        ));
        let native = scope.function(&Callback::new(|scope, _| {
            Ok(Completion::Throw(scope.dom_exception(
                DomExceptionName::OperationError,
                &[0xd800],
            )?))
        }))?;
        Ok::<_, Error>((
            scope.retain(&constructor),
            scope.retain(&prototype),
            scope.retain(&native),
        ))
    })?;
    let (target, prototype_b) = execution.in_realm(&b, |scope| {
        dom_exception::install(scope, DomExceptionLimits::default(), ())?;
        let prototype = scope.evaluate(&units("DOMException.prototype"))?;
        let target = scope.evaluate(&units(
            "function Target() {}
             Target.prototype = null;
             delete DOMException.prototype.constructor;
             Object.defineProperty(globalThis, 'DOMException', {
                get() { throw 'B DOMException global lookup'; }
             });
             Target",
        ))?;
        Ok::<_, Error>((scope.retain(&target), scope.retain(&prototype)))
    })?;
    execution.in_realm(&driver, |scope| {
        let global = scope.global()?;
        for (name, root) in [
            ("ConstructorA", &constructor),
            ("prototypeA", &prototype_a),
            ("nativeA", &native),
            ("targetB", &target),
            ("prototypeB", &prototype_b),
        ] {
            let value = scope.import(root)?;
            scope.set(&global, PropertyKey::String(&units(name)), &value)?;
        }
        Ok::<_, Error>(())
    })?;
    drop((constructor, prototype_a, native, target, prototype_b, a, b));
    execution.memory_pressure();
    assert_eq!(
        evaluate(
            &mut execution,
            &driver,
            r#"(() => {
                const e = Reflect.construct(ConstructorA, ["from A", "NotFoundError"], targetB);
                const readA = Object.getOwnPropertyDescriptor(prototypeA, "name").get;
                const readB = Object.getOwnPropertyDescriptor(prototypeB, "message").get;
                if (!Error.isError(e) || prototypeA === prototypeB
                    || Object.getPrototypeOf(e) !== prototypeB
                    || readA.call(e) !== "NotFoundError" || readB.call(e) !== "from A"
                    || e.code !== 8) return false;
                try { nativeA(); return false; }
                catch (e) {
                    return Error.isError(e) && Object.getPrototypeOf(e) === prototypeA
                        && e.name === "OperationError" && e.message === "\ud800" && e.code === 0;
                }
            })()"#,
        )?,
        units("true")
    );
    Ok(())
}
