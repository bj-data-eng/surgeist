use super::{TestResult, account, new_ledger, units};
use std::{cell::Cell, rc::Rc};
use surgeist_bindings_js::conversion::{
    ArgumentValue, declarations, trusted_policy_callback_result, types,
};
use surgeist_bindings_js::{
    Error, ObjectPrototype, Process, PropertyKey, TrustedAttributeValue, TrustedTypeKind,
};
use surgeist_ledger::{Limits, ProfileMode};

pub(super) fn run(process: &mut Process) -> TestResult {
    let realm = process.execution().create_realm()?;
    let other = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let account = account(process, &realm, &ledger, Limits::unlimited())?;
    let branded = process.execution().in_realm(&realm, |scope| {
        let callback = scope.evaluate(&units("() => ({toString(){return '\\ud800\\x00'}})"))?;
        let receiver = scope.global()?;
        let result = scope.call(&callback, &receiver, &[])?;
        for (kind, expected) in [
            (TrustedTypeKind::Html, [0xd800, 0]),
            (TrustedTypeKind::Script, [0xd800, 0]),
            (TrustedTypeKind::ScriptUrl, [0xfffd, 0]),
        ] {
            let converted = account.run(scope, |cx| cx.value(&trusted_policy_callback_result(kind), &result))?;
            assert_eq!(converted.get().as_ref().map(|data| data.as_code_units()), Some(expected.as_slice()));
        }
        let converted = account.run(scope, |cx| cx.value(&trusted_policy_callback_result(TrustedTypeKind::Script), &result))?;
        let mut admitted = false;
        let branded = scope.project_trusted_type(ObjectPrototype::Default, converted, |bytes| {
            admitted = true;
            assert!(bytes >= 2 * size_of::<u16>());
            assert_eq!(ledger.usage(ledger.root()).unwrap().live(super::Resource::BufferBytes), 2 * size_of::<u16>() as u64);
            Ok::<_, Error>(())
        })?.expect("a present callback value projects its exact brand");
        assert!(admitted);
        assert_eq!(scope.trusted_type(&branded)?.unwrap().kind(), TrustedTypeKind::Script);
        assert_eq!(ledger.usage(ledger.root()).unwrap().live(super::Resource::BufferBytes), 0);
        scope.set(&receiver, PropertyKey::String(&units("branded")), &branded)?;
        scope.evaluate(&units("branded[Symbol.toPrimitive] = () => { throw 'coerced brand' };"))?;

        let argument = declarations::element_set_attribute_0_value();
        assert_eq!(argument.index(), 1);
        let converted = account.run(scope, |cx| cx.argument(&argument, Some(&branded)))?;
        assert!(matches!(converted.get(), ArgumentValue::Present(TrustedAttributeValue::Trusted(value))
            if value.kind() == TrustedTypeKind::Script && value.as_code_units() == [0xd800, 0]));

        for source in [
            "Object.assign(Object.create(branded), {[Symbol.toPrimitive](){return 'ordinary'}})",
            "new Proxy(branded, {get(){return () => 'ordinary'}})",
            "'ordinary'",
        ] {
            let object = scope.evaluate(&units(source))?;
            assert!(scope.trusted_type(&object)?.is_none());
            let converted = account.run(scope, |cx| cx.value(&types::TRUSTED_ATTRIBUTE_VALUE, &object))?;
            assert!(matches!(converted.get(), TrustedAttributeValue::String(value) if value.as_code_units() == units("ordinary")));
        }

        for source in ["null", "undefined"] {
            let result = scope.evaluate(&units(source))?;
            let converted = account.run(scope, |cx| cx.value(&trusted_policy_callback_result(TrustedTypeKind::Html), &result))?;
            assert!(converted.get().is_none());
            let projected = scope.project_trusted_type(ObjectPrototype::Default, converted, |_| -> Result<(), Error> {
                panic!("a null policy result must never allocate a branded object");
            })?;
            assert!(projected.is_none());
        }

        let abrupt = scope.evaluate(&units("globalThis.original = {}; ({toString(){throw original}})"))?;
        let Err(failure) = account.run(scope, |cx| cx.value(&trusted_policy_callback_result(TrustedTypeKind::ScriptUrl), &abrupt)) else {
            panic!("a callback result can throw during coercion");
        };
        let original = scope.evaluate(&units("original"))?;
        let thrown = scope.import(failure.exception().expect("exact guest throw").value())?;
        assert!(scope.strict_equals(&original, &thrown)?);

        let noncallable = scope.object(ObjectPrototype::Default)?;
        assert!(scope.retain_trusted_policy_callback(&noncallable, |_| -> Result<(), Error> {
            panic!("a noncallable option cannot acquire a callback holder");
        })?.is_none());
        let callback = scope.evaluate(&units(concat!(
            "globalThis.conversions = 0; ",
            "(function(value, type, sink) { 'use strict'; ",
            "if (this !== undefined || arguments.length !== 3 || ",
            "value !== '\\ud800\\0' || type !== 'TrustedScriptURL' || ",
            "sink !== 'HTMLScriptElement src') throw new Error('invalid callback arguments'); ",
            "return {toString() { conversions++; return '\\ud800\\0'; }}; })",
        )))?;
        let released = Rc::new(Cell::new(false));
        let retained = scope.retain_trusted_policy_callback(&callback, |bytes| {
            assert!(bytes > 0);
            Ok::<_, Error>(Release(released.clone()))
        })?.expect("callable option has a holder");
        assert_eq!(retained.associated_realm().identity(), scope.realm_authority()?.identity());
        let output = account.run(scope, |cx| cx.invoke_default_policy(
            &retained, TrustedTypeKind::ScriptUrl, &[0xd800, 0], &units("HTMLScriptElement src"),
        ))?;
        assert_eq!(output.get().as_ref().unwrap().as_code_units(), [0xfffd, 0]);
        assert_eq!(ledger.usage(ledger.root()).unwrap().live(super::Resource::BufferBytes), 4);
        drop(output);
        assert_eq!(ledger.usage(ledger.root()).unwrap().live(super::Resource::BufferBytes), 0);
        let count = scope.evaluate(&units("conversions"))?;
        let once = scope.primitive(surgeist_bindings_js::Primitive::Number(1.0))?;
        assert!(scope.strict_equals(&count, &once)?);
        assert!(!released.get());
        scope.release_trusted_policy_callback(retained)?;
        assert!(released.get());

        for source in ["() => {throw original}", "() => ({toString(){throw original}})"] {
            let callback = scope.evaluate(&units(source))?;
            let retained = scope.retain_trusted_policy_callback(&callback, |_| Ok::<_, Error>(()))?.unwrap();
            let Err(failure) = account.run(scope, |cx| cx.invoke_default_policy(
                &retained, TrustedTypeKind::Html, &[], &units("HTMLIFrameElement srcdoc"),
            )) else {
                panic!("the original callback or conversion throws");
            };
            let thrown = scope.import(failure.exception().unwrap().value())?;
            assert!(scope.strict_equals(&original, &thrown)?);
            scope.release_trusted_policy_callback(retained)?;
        }
        Ok(scope.retain(&branded))
    })?;
    process.execution().in_realm(&other, |scope| {
        let original = scope.import(&branded)?;
        let argument = declarations::element_set_attribute_ns_0_value();
        assert_eq!(argument.index(), 2);
        let converted = account.run(scope, |cx| cx.argument(&argument, Some(&original)))?;
        assert!(
            matches!(converted.get(), ArgumentValue::Present(TrustedAttributeValue::Trusted(value))
            if value.kind() == TrustedTypeKind::Script && value.as_code_units() == [0xd800, 0])
        );
        Ok(())
    })?;
    Ok(())
}

struct Release(Rc<Cell<bool>>);

impl Drop for Release {
    fn drop(&mut self) {
        self.0.set(true);
    }
}
