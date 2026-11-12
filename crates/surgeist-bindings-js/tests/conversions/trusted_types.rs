use super::{TestResult, account, new_ledger, units};
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
