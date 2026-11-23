use std::cell::Cell;

use boa_gc::Gc;
use surgeist_bindings_js::{
    Callback, Completion, EngineInstructionCount, Error, Primitive, Process,
};

use super::units;

pub(super) fn engine_measurements() -> Result<(), Error> {
    let mut process = Process::bootstrap()?;
    let a = process.execution().create_realm()?;
    let b = process.execution().create_realm()?;
    observe_intrinsic_collection(&mut process, &a)?;

    assert_eq!(
        process.instruction_count(),
        EngineInstructionCount::Disabled
    );
    let (guest_a, throwing) = process.execution().in_realm(&a, |scope| {
        let ordinary = scope.evaluate(&units("() => 17"))?;
        let throwing = scope.evaluate(&units("() => { throw 23; }"))?;
        Ok((scope.retain(&ordinary), scope.retain(&throwing)))
    })?;
    let guest_b = process.execution().in_realm(&b, |scope| {
        let function = scope.evaluate(&units("() => 42"))?;
        Ok(scope.retain(&function))
    })?;
    assert_eq!(
        process.instruction_count(),
        EngineInstructionCount::Disabled
    );
    process.enable_instruction_counting()?;

    process.execution().in_realm(&a, |scope| {
        let native = scope.function(&Callback::new(|_, _| Ok(Completion::Undefined)))?;
        let receiver = scope.primitive(Primitive::Undefined)?;
        scope.call(&native, &receiver, &[])?;
        Ok(())
    })?;
    assert_eq!(
        process.instruction_count(),
        EngineInstructionCount::Exact(0)
    );

    // The pinned compiler emits StoreInt8, Push, Pop, SetAccumulator,
    // CheckReturn and Return for the reached literal-arrow path.
    const LITERAL_ARROW_DISPATCHES: u64 = 6;
    process.execution().in_realm(&a, |scope| {
        let function = scope.import(&guest_a)?;
        let receiver = scope.primitive(Primitive::Undefined)?;
        let returned = scope.call(&function, &receiver, &[])?;
        assert_eq!(scope.to_utf16(&returned)?, units("17"));
        Ok(())
    })?;
    assert_eq!(
        process.instruction_count(),
        EngineInstructionCount::Exact(LITERAL_ARROW_DISPATCHES)
    );

    process.execution().in_realm(&b, |scope| {
        let reenter = scope.function(&Callback::new(move |scope, _| {
            let function = scope.import(&guest_a)?;
            let receiver = scope.primitive(Primitive::Undefined)?;
            Ok(Completion::Return(scope.call(&function, &receiver, &[])?))
        }))?;
        let receiver = scope.primitive(Primitive::Undefined)?;
        let returned = scope.call(&reenter, &receiver, &[])?;
        assert_eq!(scope.to_utf16(&returned)?, units("17"));
        let future = scope.import(&guest_b)?;
        scope.enqueue_microtask(&future)
    })?;
    assert_eq!(
        process.instruction_count(),
        EngineInstructionCount::Exact(2 * LITERAL_ARROW_DISPATCHES)
    );
    process.execution().checkpoint()?;
    assert_eq!(
        process.instruction_count(),
        EngineInstructionCount::Exact(3 * LITERAL_ARROW_DISPATCHES)
    );

    let result = process.execution().in_realm(&a, |scope| {
        let function = scope.import(&throwing)?;
        let receiver = scope.primitive(Primitive::Undefined)?;
        scope.call(&function, &receiver, &[]).map(|_| ())
    });
    assert!(matches!(result, Err(Error::Exception(_))));
    // The abrupt path reaches the literal load and Throw.
    let reached = EngineInstructionCount::Exact(3 * LITERAL_ARROW_DISPATCHES + 2);
    assert_eq!(process.instruction_count(), reached);
    process.enable_instruction_counting()?;
    assert_eq!(process.instruction_count(), reached);
    let before_refusal = process.execution().observe_heap()?;

    let refusal = process.execution().in_realm(&a, |scope| {
        let native = scope.function(&Callback::new(|_, _| Err(Error::ExpectedFunction)))?;
        let receiver = scope.primitive(Primitive::Undefined)?;
        scope.call(&native, &receiver, &[]).map(|_| ())
    });
    assert!(matches!(refusal, Err(Error::ExpectedFunction)));
    assert_eq!(process.instruction_count(), reached);
    let entered = Cell::new(false);
    let refused = process.execution().in_realm(&b, |_| {
        entered.set(true);
        Ok(())
    });
    assert!(matches!(refused, Err(Error::ExpectedFunction)));
    assert!(matches!(
        process.enable_instruction_counting(),
        Err(Error::ExpectedFunction)
    ));
    assert!(matches!(
        process.execution().observe_heap(),
        Err(Error::ExpectedFunction)
    ));
    assert!(!entered.get());
    assert_eq!(process.instruction_count(), reached);

    drop(process);
    let mut successor = Process::bootstrap()?;
    assert_eq!(
        successor.instruction_count(),
        EngineInstructionCount::Disabled
    );
    let heap = successor.execution().observe_heap()?;
    assert!(
        heap.collections_entered
            .expect("exact thread collection total")
            >= before_refusal
                .collections_entered
                .expect("exact prior collection total")
    );
    assert!(
        heap.cumulative_shallow_bytes
            .expect("exact allocation total")
            > before_refusal
                .cumulative_shallow_bytes
                .expect("exact prior allocation total")
    );
    successor.enable_instruction_counting()?;
    assert_eq!(
        successor.instruction_count(),
        EngineInstructionCount::Exact(0)
    );
    assert_eq!(
        successor.execution().observe_heap()?.collections_entered,
        heap.collections_entered
    );
    Ok(())
}

fn observe_intrinsic_collection(
    process: &mut Process,
    realm: &surgeist_bindings_js::Realm,
) -> Result<(), Error> {
    // The fresh Boa collector's pinned initial cutoff is 1 MiB. Populate
    // through its normal allocation path until the next allocation will
    // collect. The pending resolver performs that allocation.
    const INITIAL_CUTOFF: usize = 1_048_576;
    const BODY_BYTES: usize = 8192;
    let mut execution = process.execution();
    let start = execution.observe_heap()?;
    assert_eq!(start.collections_entered, Some(0));
    assert!(start.used_bytes <= INITIAL_CUTOFF);
    let mut retained = Vec::new();
    let mut filled = start;
    while filled.used_bytes <= INITIAL_CUTOFF {
        assert!(retained.len() <= INITIAL_CUTOFF / BODY_BYTES);
        retained.push(Gc::new([0_u8; BODY_BYTES]));
        filled = execution.observe_heap()?;
    }
    assert_eq!(filled.collections_entered, start.collections_entered);

    execution.in_realm(realm, |scope| {
        let resolver = scope.promise_resolver()?;
        let promise = scope.resolver_promise(&resolver)?;
        assert_eq!(
            scope.kind(&promise)?,
            surgeist_bindings_js::ValueKind::Object
        );
        Ok(())
    })?;
    let collected = execution.observe_heap()?;
    let entered = filled
        .collections_entered
        .expect("exact collection count before intrinsic allocation")
        + 1;
    assert_eq!(collected.collections_entered, Some(entered));
    let cumulative = collected
        .cumulative_shallow_bytes
        .expect("exact bytes after intrinsic allocation");
    assert!(
        cumulative
            > filled
                .cumulative_shallow_bytes
                .expect("exact allocated total")
    );
    assert_eq!(execution.observe_heap()?, collected);

    drop(retained);
    execution.memory_pressure();
    let reclaimed = execution.observe_heap()?;
    assert_eq!(reclaimed.collections_entered, Some(entered + 1));
    assert_eq!(reclaimed.cumulative_shallow_bytes, Some(cumulative));
    assert!(reclaimed.used_bytes < collected.used_bytes);
    Ok(())
}
