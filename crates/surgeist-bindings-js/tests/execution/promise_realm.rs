use super::units;
use surgeist_bindings_js::{Error, Process, PropertyKey};

pub(super) fn promise_resolver_rejects_self_resolution_in_its_original_realm(
    process: &mut Process,
) -> Result<(), Error> {
    let mut execution = process.execution();
    let original = execution.create_realm()?;
    let other = execution.create_realm()?;
    let (resolver, promise, original_type_error) = execution.in_realm(&original, |scope| {
        let resolver = scope.promise_resolver()?;
        let promise = scope.resolver_promise(&resolver)?;
        let type_error = scope.evaluate(&units("TypeError"))?;
        Ok((resolver, scope.retain(&promise), scope.retain(&type_error)))
    })?;
    execution.in_realm(&other, |scope| {
        let global = scope.global()?;
        let promise = scope.import(&promise)?;
        let original_type_error = scope.import(&original_type_error)?;
        scope.set(
            &global,
            PropertyKey::String(&units("originalPromise")),
            &promise,
        )?;
        scope.set(
            &global,
            PropertyKey::String(&units("originalTypeError")),
            &original_type_error,
        )?;
        scope.evaluate(&units(
            "globalThis.rejections = 0;
             globalThis.correctRealm = false;
             originalPromise.catch(error => {
                 rejections++;
                 correctRealm = error instanceof originalTypeError && !(error instanceof TypeError);
             });",
        ))?;
        scope.resolve_promise(resolver, &promise)?;
        let pending = scope.evaluate(&units("rejections === 0"))?;
        assert!(
            scope.boolean(&pending)?,
            "rejection handler ran before the checkpoint"
        );
        Ok(())
    })?;
    execution.checkpoint()?;
    execution.in_realm(&other, |scope| {
        let rejected = scope.evaluate(&units("rejections === 1 && correctRealm"))?;
        assert!(
            scope.boolean(&rejected)?,
            "expected one rejection with the resolver realm's TypeError"
        );
        Ok(())
    })
}
