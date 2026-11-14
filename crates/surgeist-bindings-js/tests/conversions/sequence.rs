use std::cell::{Cell, RefCell};
use std::mem;
use std::rc::Rc;

use surgeist_bindings_js::conversion::{
    Converted, Resource, ResourceTable, Work, WorkTable, declarations, types,
};
use surgeist_bindings_js::{
    Callback, Completion, DomString, EngineKind, Error, Primitive, Process, PropertyKey,
};
use surgeist_ledger::{EventFilter, Limits, ProfileMode};

use super::{TestResult, account, new_ledger, units};

type Strings = Converted<Vec<Converted<DomString>>>;

fn cached_iterator_and_next(process: &mut Process) -> TestResult {
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let account = account(process, &realm, &ledger, Limits::unlimited())?;
    let retained: Rc<RefCell<Option<Strings>>> = Rc::new(RefCell::new(None));
    let converter = account.clone();
    let output = retained.clone();
    let collect = Callback::new(move |scope, args| {
        let input = args.get(0).ok_or(Error::BackendUnavailable)?;
        let value = converter.run(scope, |cx| cx.value(&types::DOM_STRING.sequence(), input))?;
        output.replace(Some(value));
        Ok(Completion::Undefined)
    });
    let nested = Rc::new(Cell::new(0));
    let (calls, converter) = (nested.clone(), account.clone());
    let reenter = Callback::new(move |scope, args| {
        let input = args.get(0).ok_or(Error::BackendUnavailable)?;
        let result = converter.run(scope, |cx| cx.value(&types::BOOLEAN, input))?;
        calls.set(calls.get() + 1);
        Ok(Completion::Return(
            scope.primitive(Primitive::Boolean(*result.get()))?,
        ))
    });
    process.execution().in_realm(&realm, |scope| {
        let global = scope.global()?;
        for (name, callback) in [("collect", &collect), ("reenter", &reenter)] {
            let function = scope.function(callback)?;
            scope.set(&global, PropertyKey::String(&units(name)), &function)?;
        }
        let result = scope.evaluate(&units(r#"(() => {
            const iteratorSymbol = Symbol.iterator;
            globalThis.Symbol = null;
            const trace = [];
            let i = 0;
            const backing = {
                get next() {
                    trace.push('getNext');
                    return function() {
                        const n = i++;
                        trace.push('next' + n);
                        return {
                            get done() { trace.push('done' + n); return n === 2; },
                            get value() {
                                trace.push('value' + n);
                                if (n === 2) throw 'must not read terminal value';
                                return {toString() {
                                    trace.push('convert' + n);
                                    Object.defineProperty(iterator, 'next', {value() {throw 'new next';}});
                                    Object.defineProperty(iterable, iteratorSymbol, {value() {throw 'new method';}});
                                    if (!reenter(true)) throw 'reentry';
                                    return n ? 'b' : 'a';
                                }};
                            }
                        };
                    };
                },
                get return() { throw 'must not close'; }
            };
            const iterator = new Proxy(backing, {
                get(target, key, receiver) {
                    if (key !== 'next') throw 'unexpected iterator read';
                    return Reflect.get(target, key, receiver);
                },
                getOwnPropertyDescriptor() {throw 'unexpected descriptor read';}
            });
            const iterable = new Proxy({
                get [iteratorSymbol]() {
                    trace.push('getIterator');
                    return function() {trace.push('method:' + (this === iterable)); return iterator;};
                }
            }, {
                get(target, key, receiver) {
                    if (key !== iteratorSymbol) throw 'unexpected iterable read';
                    return Reflect.get(target, key, receiver);
                },
                ownKeys() {throw 'unexpected enumeration';}
            });
            collect(iterable);
            return trace.join(',');
        })()"#))?;
        assert_eq!(scope.to_utf16(&result)?, units("getIterator,method:true,getNext,next0,done0,value0,convert0,next1,done1,value1,convert1,next2,done2"));
        Ok(())
    })?;
    assert_eq!(nested.get(), 2);
    {
        let values = retained.borrow();
        let values = values.as_ref().expect("retained sequence");
        assert_eq!(
            values
                .get()
                .iter()
                .map(|v| v.get().as_code_units())
                .collect::<Vec<_>>(),
            [&[97][..], &[98][..]]
        );
        assert_eq!(
            ledger.usage(ledger.root())?.live(Resource::BufferBytes),
            4 * mem::size_of::<Converted<DomString>>() as u64 + 4
        );
    }
    drop(retained.replace(None));
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    assert_eq!(account.cleanup_status().failures(), Some(0));
    Ok(())
}

fn abrupts_and_no_close(process: &mut Process) -> TestResult {
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let account = account(process, &realm, &ledger, Limits::unlimited())?;
    let completed = Rc::new(Cell::new(0));
    let (count, converter) = (completed.clone(), account.clone());
    let collect = Callback::new(move |scope, args| {
        let input = args.get(0).ok_or(Error::BackendUnavailable)?;
        converter.run(scope, |cx| cx.value(&types::DOM_STRING.sequence(), input))?;
        count.set(count.get() + 1);
        Ok(Completion::Undefined)
    });
    process.execution().in_realm(&realm, |scope| {
        let global = scope.global()?;
        let function = scope.function(&collect)?;
        scope.set(&global, PropertyKey::String(&units("collect")), &function)?;
        let result = scope.evaluate(&units(r#"(() => {
            const TypeErrorIntrinsic = TypeError;
            globalThis.TypeError = function() {throw 'wrong constructor';};
            for (const input of [null, undefined, '', 1, {}, {[Symbol.iterator]: null}, {[Symbol.iterator]: 1}, {[Symbol.iterator]() {return 1;}}]) {
                let threw = false;
                try { collect(input); } catch(e) { threw = e instanceof TypeErrorIntrinsic; }
                if (!threw) return false;
            }
            for (const phase of ['method', 'next-get', 'next-call', 'next-throw', 'result', 'done', 'value', 'convert']) {
                const primary = {};
                let nextCalls = 0, closeReads = 0;
                const iterator = {
                    get return() {closeReads++; throw 'unexpected close';},
                    get next() {
                        if (phase === 'next-get') throw primary;
                        if (phase === 'next-call') return 42;
                        return function() {
                            nextCalls++;
                            if (nextCalls === 1) return {done: false, value: 'prefix'};
                            if (phase === 'next-throw') throw primary;
                            if (phase === 'result') return 1;
                            return {
                                get done() {if (phase === 'done') throw primary; return false;},
                                get value() {
                                    if (phase === 'value') throw primary;
                                    return {toString() {throw primary;}};
                                }
                            };
                        };
                    }
                };
                const iterable = {get [Symbol.iterator]() {
                    if (phase === 'method') throw primary;
                    return () => iterator;
                }};
                let same = false;
                try {collect(iterable);} catch(e) {
                    same = ['next-call', 'result'].includes(phase) ? e instanceof TypeErrorIntrinsic : e === primary;
                }
                if (!same || closeReads !== 0 || nextCalls > 2) return false;
            }
            return true;
        })()"#))?;
        assert!(scope.boolean(&result)?);
        scope.check_active()?;
        Ok(())
    })?;
    assert_eq!(completed.get(), 0);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    assert_eq!(account.cleanup_status().failures(), Some(0));
    Ok(())
}

fn nested_values_and_dictionary_steps(process: &mut Process) -> TestResult {
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let account = account(process, &realm, &ledger, Limits::unlimited())?;
    let retained = process.execution().in_realm(&realm, |scope| {
        let input = scope.evaluate(&units("[['A', null], ['BC']]"))?;
        let output = account.run(scope, |cx| {
            cx.value(
                &types::DOM_STRING
                    .nullable()
                    .sequence()
                    .sequence()
                    .nullable(),
                &input,
            )
        })?;
        let outer = output.get().as_ref().expect("outer");
        assert_eq!(outer.len(), 2);
        assert_eq!(
            outer[0].get()[0].get().as_ref().expect("A").as_code_units(),
            [65]
        );
        assert!(outer[0].get()[1].get().is_none());
        assert_eq!(
            outer[1].get()[0]
                .get()
                .as_ref()
                .expect("BC")
                .as_code_units(),
            [66, 67]
        );
        Ok(output)
    })?;
    let expected = 4 * mem::size_of::<Converted<Vec<Converted<Option<DomString>>>>>() as u64
        + 8 * mem::size_of::<Converted<Option<DomString>>>() as u64
        + 6;
    assert_eq!(
        ledger.usage(ledger.root())?.live(Resource::BufferBytes),
        expected
    );
    drop(realm);
    process.execution().memory_pressure();
    assert_eq!(
        ledger.usage(ledger.root())?.live(Resource::BufferBytes),
        expected
    );
    drop(retained);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);

    let realm = process.execution().create_realm()?;
    let converter = account.clone();
    let callback = Callback::new(move |scope, args| {
        let input = args.get(0).ok_or(Error::BackendUnavailable)?;
        let values = converter.run(scope, |cx| {
            cx.value(&declarations::event_init().sequence(), input)
        })?;
        assert_eq!(values.get().len(), 2);
        assert!(values.get()[0].get().bubbles);
        assert!(!values.get()[1].get().bubbles);
        Ok(Completion::Undefined)
    });
    process.execution().in_realm(&realm, |scope| {
        let global = scope.global()?;
        let function = scope.function(&callback)?;
        scope.set(&global, PropertyKey::String(&units("dicts")), &function)?;
        let result = scope.evaluate(&units(
            r#"(() => {
            const trace = [];
            let i = 0;
            dicts({[Symbol.iterator]() { return {next() {
                const n = i++; trace.push('next' + n);
                return {done: n === 2, value: {
                    get bubbles() {trace.push('b' + n); return n === 0;},
                    get cancelable() {trace.push('c' + n); return false;},
                    get composed() {trace.push('d' + n); return false;}
                }};
            }};}});
            return trace.join(',');
        })()"#,
        ))?;
        assert_eq!(
            scope.to_utf16(&result)?,
            units("next0,b0,c0,d0,next1,b1,c1,d1,next2")
        );
        Ok(())
    })?;
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    Ok(())
}

fn empty_and_profiling(process: &mut Process) -> TestResult {
    let realm = process.execution().create_realm()?;
    for mode in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 32,
            filter: EventFilter::ScopeName("excluded"),
        },
    ] {
        let ledger = new_ledger(mode)?;
        let account = account(process, &realm, &ledger, Limits::unlimited())?;
        process.execution().in_realm(&realm, |scope| {
            let empty = scope.evaluate(&units("[]"))?;
            let value = account.run(scope, |cx| cx.value(&types::DOM_STRING.sequence(), &empty))?;
            assert!(value.get().is_empty());
            assert_eq!(
                ledger
                    .usage(ledger.root())
                    .expect("usage")
                    .peak(Resource::BufferBytes),
                0
            );
            drop(value);
            let input = scope.evaluate(&units("['a', 'b', 'c', 'd', 'e']"))?;
            let output =
                account.run(scope, |cx| cx.value(&types::DOM_STRING.sequence(), &input))?;
            let values = output
                .get()
                .iter()
                .map(|v| v.get().as_code_units())
                .collect::<Vec<_>>();
            assert_eq!(
                values,
                [&[97][..], &[98][..], &[99][..], &[100][..], &[101][..]]
            );
            let usage = ledger.usage(ledger.root()).expect("usage");
            assert_eq!(usage.work(Work::IteratorOperation), 25);
            assert_eq!(usage.work(Work::SequenceItem), 5);
            assert_eq!(usage.work(Work::SequenceMove), 4);
            assert_eq!(usage.work(Work::Value), 7);
            assert_eq!(
                usage.live(Resource::BufferBytes),
                8 * mem::size_of::<Converted<DomString>>() as u64 + 10
            );
            assert_eq!(
                usage.peak(Resource::BufferBytes),
                12 * mem::size_of::<Converted<DomString>>() as u64 + 10
            );
            Ok(())
        })?;
        assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Cut {
    Exact,
    Storage,
    Iterator,
    Item,
    Movement,
}

fn bounded_growth(cut: Cut) -> TestResult {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let peak = 12 * mem::size_of::<Converted<DomString>>() as u64 + 10;
    let limits = Limits::new(
        WorkTable {
            Value: 6,
            CodeUnit: 5,
            EnumerationLookup: 0,
            Argument: 0,
            DictionaryMember: 0,
            IteratorOperation: if matches!(cut, Cut::Iterator) { 19 } else { 20 },
            SequenceItem: if matches!(cut, Cut::Item) { 4 } else { 5 },
            SequenceMove: if matches!(cut, Cut::Movement) { 3 } else { 4 },
        },
        ResourceTable {
            BufferBytes: if matches!(cut, Cut::Storage) {
                peak - 1
            } else {
                peak
            },
        },
    );
    let account = account(&mut process, &realm, &ledger, limits)?;
    let events = Rc::new(RefCell::new(Vec::new()));
    let seen = events.clone();
    let mark = Callback::new(move |scope, args| {
        let item = args.get(0).ok_or(Error::BackendUnavailable)?;
        let marker = scope.to_utf16(item)?;
        seen.borrow_mut().push(marker);
        Ok(Completion::Undefined)
    });
    let accepted = Rc::new(Cell::new(0));
    let count = accepted.clone();
    let converter = account.clone();
    let callback = Callback::new(move |scope, args| {
        let input = args.get(0).ok_or(Error::BackendUnavailable)?;
        converter.run(scope, |cx| cx.value(&types::DOM_STRING.sequence(), input))?;
        scope.check_active()?;
        count.set(count.get() + 1);
        Ok(Completion::Undefined)
    });
    let result = process.execution().in_realm(&realm, |scope| {
        let global = scope.global()?;
        for (name, callback) in [("collect", &callback), ("mark", &mark)] {
            let function = scope.function(callback)?;
            scope.set(&global, PropertyKey::String(&units(name)), &function)?;
        }
        scope.evaluate(&units(
            r#"(() => {
            let i = 0;
            const iterator = {next() {
                const n = i++; mark('next' + n);
                return {get done() {mark('done' + n); return n === 5;},
                        get value() {mark('value' + n); return 'x';}};
            }, get return() {mark('CLOSE'); throw 'unexpected';}};
            try {collect({[Symbol.iterator]() {return iterator;}});} catch (_) {}
        })()"#,
        ))?;
        Ok(())
    });
    let observed = events.borrow();
    assert!(!observed.iter().any(|marker| marker == &units("CLOSE")));
    if matches!(cut, Cut::Exact) {
        result?;
        assert_eq!(accepted.get(), 1);
        assert_eq!(
            observed.last().map(|marker| &**marker),
            Some(&units("done5"))
        );
        assert_eq!(
            ledger.usage(ledger.root())?.peak(Resource::BufferBytes),
            peak
        );
    } else {
        assert!(matches!(result, Err(Error::ConversionLedger(_))));
        assert_eq!(accepted.get(), 0);
        let last = if matches!(cut, Cut::Iterator) {
            "next5"
        } else if matches!(cut, Cut::Item) {
            "done4"
        } else {
            "value4"
        };
        assert_eq!(observed.last().map(|marker| &**marker), Some(&units(last)));
    }
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    assert_eq!(account.cleanup_status().failures(), Some(0));
    Ok(())
}

pub(super) fn run(process: &mut Process) -> TestResult {
    cached_iterator_and_next(process)?;
    abrupts_and_no_close(process)?;
    nested_values_and_dictionary_steps(process)?;
    empty_and_profiling(process)
}

fn retained_nested_output_outlives_heap() -> TestResult {
    let ledger = new_ledger(ProfileMode::Off)?;
    let output = {
        let mut process = Process::bootstrap(EngineKind::V8)?;
        let realm = process.execution().create_realm()?;
        let account = account(&mut process, &realm, &ledger, Limits::unlimited())?;
        process.execution().in_realm(&realm, |scope| {
            let input = scope.evaluate(&units("[['a', 'b']]"))?;
            account
                .run(scope, |cx| {
                    cx.value(&types::DOM_STRING.sequence().sequence(), &input)
                })
                .map_err(Error::from)
        })?
    };
    assert!(ledger.usage(ledger.root())?.live(Resource::BufferBytes) > 0);
    assert_eq!(output.get()[0].get()[1].get().as_code_units(), [98]);
    drop(output);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    Ok(())
}

fn callback_unwind_releases_nested_output() -> TestResult {
    let mut process = Process::bootstrap(EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let account = account(&mut process, &realm, &ledger, Limits::unlimited())?;
    let converter = account.clone();
    let observe = ledger.clone();
    let callback = Callback::new(move |scope, args| {
        let input = args.get(0).ok_or(Error::BackendUnavailable)?;
        let _retained = converter.run(scope, |cx| {
            cx.value(&types::DOM_STRING.sequence().sequence(), input)
        })?;
        assert!(
            observe
                .usage(observe.root())
                .expect("usage")
                .live(Resource::BufferBytes)
                > 0
        );
        panic!("sequence consumer body panic");
    });
    let result = process.execution().in_realm(&realm, |scope| {
        let function = scope.function(&callback)?;
        let receiver = scope.global()?;
        let input = scope.evaluate(&units("[['a', 'b']]"))?;
        scope.call(&function, &receiver, &[input]).map(|_| ())
    });
    assert!(matches!(result, Err(Error::CallbackPanicked)));
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    assert_eq!(account.cleanup_status().failures(), Some(0));
    assert!(matches!(
        process.execution().in_realm(&realm, |_| Ok(())),
        Err(Error::CallbackPanicked)
    ));
    Ok(())
}

pub(super) fn independent_owners() -> TestResult {
    for cut in [
        Cut::Exact,
        Cut::Storage,
        Cut::Iterator,
        Cut::Item,
        Cut::Movement,
    ] {
        bounded_growth(cut)?;
    }
    retained_nested_output_outlives_heap()?;
    callback_unwind_releases_nested_output()
}
