use std::cell::{Cell, RefCell};
use std::rc::Rc;

use surgeist_bindings_js::conversion::{
    ArgumentValue, Converted, FailureKind, IntegerType, Resource, ResourceTable, Rule, Work,
    WorkTable, declarations, types,
};
use surgeist_bindings_js::declarations::EventInit;
use surgeist_bindings_js::{
    Callback, Completion, DomString, Error, IntegerKind, IntegerMode, IntegerValue, Primitive,
    Process, PropertyKey,
};
use surgeist_ledger::{Limits, ProfileMode};

use super::{TestResult, account, new_ledger, units};

fn dictionary_reads_and_reentry(process: &mut Process) -> TestResult {
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Summary { timing: false })?;
    let account = account(process, &realm, &ledger, Limits::unlimited())?;
    let accepted = Rc::new(Cell::new(0));
    let observed = Rc::new(Cell::new(EventInit::default()));
    let (count, result, converter) = (accepted.clone(), observed.clone(), account.clone());
    let callback = Callback::new(move |scope, arguments| {
        let argument = declarations::event_constructor_0_event_init_dict();
        let output = converter.run(scope, |cx| {
            cx.argument(&argument, arguments.get(argument.index()))
        })?;
        let ArgumentValue::Present(value) = output.get() else {
            return Err(Error::BackendUnavailable);
        };
        scope.check_active()?;
        result.set(*value);
        count.set(count.get() + 1);
        Ok(Completion::Return(scope.primitive(Primitive::Boolean(
            value.bubbles && value.cancelable && value.composed,
        ))?))
    });
    process.execution().in_realm(&realm, |scope| {
        let global = scope.global()?;
        let function = scope.function(&callback)?;
        scope.set(&global, PropertyKey::String(&units("readInit")), &function)?;
        let result = scope.evaluate(&units(
            r#"
            globalThis.trace = [];
            const input = Object.create({
                get bubbles() {
                    trace.push('bubbles');
                    Object.defineProperty(input, 'cancelable', {get() {
                        trace.push('cancelable'); return true;
                    }});
                    readInit('nested', null);
                    return {valueOf() { throw 'boolean must not coerce'; }};
                },
                get composed() { trace.push('composed'); return true; }
            });
            const proxy = new Proxy(input, {
                get(target, key, receiver) {
                    trace.push('get:' + key);
                    return Reflect.get(target, key, receiver);
                },
                ownKeys() { throw 'dictionary must not enumerate'; },
                getOwnPropertyDescriptor() { throw 'dictionary must not inspect descriptors'; }
            });
            readInit('type', proxy) + '|' + trace.join(',')
        "#,
        ))?;
        assert_eq!(
            scope.to_utf16(&result)?,
            units("true|get:bubbles,bubbles,get:cancelable,cancelable,get:composed,composed")
        );
        assert_eq!(accepted.get(), 2);
        assert_eq!(
            observed.get(),
            EventInit {
                bubbles: true,
                cancelable: true,
                composed: true
            }
        );
        let result = scope.evaluate(&units(
            r#"
            const OriginalTypeError = TypeError;
            globalThis.TypeError = function() { throw 'poisoned global'; };
            const primary = {identity: 'member getter'};
            let original = false;
            try { readInit('type', {get bubbles() {throw primary;}}); }
            catch (error) { original = error === primary; }
            let standard = false;
            try { readInit('type', 1); }
            catch (error) { standard = error instanceof OriginalTypeError; }
            original && standard
        "#,
        ))?;
        assert!(scope.boolean(&result)?);
        for source in [
            "readInit('type')",
            "readInit('type', undefined)",
            "readInit('type', null)",
        ] {
            let result = scope.evaluate(&units(source))?;
            assert!(!scope.boolean(&result)?);
            assert_eq!(observed.get(), EventInit::default());
        }
        let empty = scope.evaluate(&units(
            "({bubbles: undefined, cancelable: null, composed: false})",
        ))?;
        let dictionary = account.run(scope, |cx| cx.value(&declarations::event_init(), &empty))?;
        assert_eq!(*dictionary.get(), EventInit::default());
        // Supplied objects are truthy; no Boolean valueOf/toString runs.
        let object = scope.evaluate(&units("({capture: {toString() {throw 1}}})"))?;
        let options = account.run(scope, |cx| {
            cx.value(&declarations::event_listener_options(), &object)
        })?;
        assert!(options.get().capture);
        Ok(())
    })?;
    assert_eq!(accepted.get(), 5);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    assert_eq!(account.cleanup_status().failures(), Some(0));
    Ok(())
}

fn nullable_and_declared_arguments(process: &mut Process) -> TestResult {
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let account = account(process, &realm, &ledger, Limits::unlimited())?;
    let retained: Rc<RefCell<Option<Converted<Option<DomString>>>>> = Rc::new(RefCell::new(None));
    let (output, converter) = (retained.clone(), account.clone());
    let callback = Callback::new(move |scope, arguments| {
        let undefined = scope.primitive(Primitive::Undefined)?;
        let input = arguments.get(0).unwrap_or(&undefined);
        let value = converter.run(scope, |cx| cx.value(&types::DOM_STRING.nullable(), input))?;
        output.replace(Some(value));
        Ok(Completion::Undefined)
    });
    let converter = account.clone();
    let title_callback = Callback::new(move |scope, arguments| {
        let ty = declarations::dom_implementation_create_html_document_0_title();
        let title = converter.run(scope, |cx| cx.argument(&ty, arguments.get(ty.index())))?;
        let result = match title.get() {
            ArgumentValue::Missing => scope.string(&[109, 105, 115, 115, 105, 110, 103])?,
            ArgumentValue::Present(text) => scope.string(text.as_code_units())?,
        };
        Ok(Completion::Return(result))
    });
    process.execution().in_realm(&realm, |scope| {
        let global = scope.global()?;
        let function = scope.function(&callback)?;
        scope.set(&global, PropertyKey::String(&units("nullableText")), &function)?;
        scope.evaluate(&units("nullableText('\\uD800x')"))?;
        assert_eq!(retained.borrow().as_ref().expect("output").get().as_ref().expect("text").as_code_units(), [0xd800, 120]);
        assert_eq!(ledger.usage(ledger.root()).expect("ledger").live(Resource::BufferBytes), 4);
        scope.evaluate(&units("nullableText(undefined)"))?;
        assert!(retained.borrow().as_ref().expect("output").get().is_none());
        assert_eq!(ledger.usage(ledger.root()).expect("ledger").live(Resource::BufferBytes), 0);
        scope.evaluate(&units("nullableText(null)"))?;
        assert!(retained.borrow().as_ref().expect("output").get().is_none());
        let title_function = scope.function(&title_callback)?;
        scope.set(&global, PropertyKey::String(&units("declaredTitle")), &title_function)?;
        let result = scope.evaluate(&units("[declaredTitle(), declaredTitle(undefined), declaredTitle(null), declaredTitle('')].join('|')"))?;
        assert_eq!(scope.to_utf16(&result)?, units("missing|missing|null|"));
        let result = scope.evaluate(&units("let titleCoercions = 0; declaredTitle({toString() { titleCoercions++; return '\\uD800'; }}); titleCoercions"))?;
        assert_eq!(scope.to_utf16(&result)?, units("1"));
        let undefined = scope.primitive(Primitive::Undefined)?;
        let null = scope.primitive(Primitive::Null)?;
        let value = account.run(scope, |cx| cx.value(&types::UNDEFINED.nullable(), &undefined))?;
        assert_eq!(*value.get(), Some(()));
        let value = account.run(scope, |cx| cx.value(&types::UNDEFINED.nullable(), &null))?;
        assert_eq!(*value.get(), None);
        let mode = scope.string(&units("open"))?;
        let value = account.run(scope, |cx| {
            cx.value(&declarations::shadow_root_mode().nullable(), &mode)
        })?;
        assert_eq!(*value.get(), Some(surgeist_bindings_js::declarations::ShadowRootMode::Open));
        let value = account.run(scope, |cx| {
            cx.value(&declarations::shadow_root_mode().nullable(), &null)
        })?;
        assert_eq!(*value.get(), None);
        let value = account.run(scope, |cx| cx.value(&IntegerType::new(IntegerKind::Long, IntegerMode::Wrap).nullable(), &null))?;
        assert_eq!(*value.get(), None);

        let title = declarations::dom_implementation_create_html_document_0_title();
        assert_eq!(title.index(), 0);
        assert_eq!(title.declaration(), "DOMImplementation.createHTMLDocument:0/title");
        for input in [None, Some(&undefined)] {
            let value = account.run(scope, |cx| cx.argument(&title, input))?;
            assert!(matches!(value.get(), ArgumentValue::Missing));
        }
        let value = account.run(scope, |cx| cx.argument(&title, Some(&null)))?;
        assert!(matches!(value.get(), ArgumentValue::Present(text) if text.as_code_units() == units("null")));
        let data = declarations::text_constructor_0_data();
        for input in [None, Some(&undefined)] {
            let value = account.run(scope, |cx| cx.argument(&data, input))?;
            assert!(matches!(value.get(), ArgumentValue::Present(text) if text.as_code_units().is_empty()));
        }
        let options = declarations::node_get_root_node_0_options();
        for input in [None, Some(&undefined), Some(&null)] {
            let value = account.run(scope, |cx| cx.argument(&options, input))?;
            assert!(matches!(value.get(), ArgumentValue::Present(options) if !options.composed));
        }
        let flag = declarations::node_clone_node_0_subtree();
        let value = account.run(scope, |cx| cx.argument(&flag, None))?;
        assert_eq!(*value.get(), ArgumentValue::Present(false));
        let mask = declarations::document_create_node_iterator_0_what_to_show();
        let value = account.run(scope, |cx| cx.argument(&mask, None))?;
        assert_eq!(*value.get(), ArgumentValue::Present(IntegerValue::UnsignedLong(u32::MAX)));
        let namespace = declarations::node_lookup_prefix_0_namespace();
        for input in [Some(&undefined), Some(&null)] {
            let value = account.run(scope, |cx| cx.argument(&namespace, input))?;
            assert!(matches!(value.get(), ArgumentValue::Present(None)));
        }
        let failure = account.run(scope, |cx| cx.argument(&namespace, None)).err().expect("required missing");
        assert_eq!(failure.kind(), FailureKind::TypeError(Rule::RequiredArgument("Node.lookupPrefix:0/namespace")));
        scope.check_active()?;
        Ok(())
    })?;
    retained.replace(None);
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    Ok(())
}

fn dictionary_admission(short: bool, value_cut: bool) -> TestResult {
    let mut process = Process::bootstrap(surgeist_bindings_js::EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let limits = Limits::new(
        WorkTable {
            Value: if value_cut { 1 } else { 4 },
            CodeUnit: 0,
            EnumerationLookup: 0,
            Argument: 0,
            DictionaryMember: if short { 2 } else { 3 },
            IteratorOperation: 0,
            SequenceItem: 0,
            SequenceMove: 0,
        },
        ResourceTable { BufferBytes: 0 },
    );
    let account = account(&mut process, &realm, &ledger, limits)?;
    let effects = Rc::new(Cell::new(0));
    let getters = Rc::new(RefCell::new(Vec::new()));
    let seen = getters.clone();
    let mark = Callback::new(move |scope, args| {
        let value = scope.integer(
            args.get(0).ok_or(Error::BackendUnavailable)?,
            IntegerKind::Long,
            IntegerMode::Wrap,
        )?;
        seen.borrow_mut().push(value);
        Ok(Completion::Undefined)
    });
    let count = effects.clone();
    let callback = Callback::new(move |scope, arguments| {
        account.run(scope, |cx| {
            cx.value(
                &declarations::event_init(),
                arguments.get(0).expect("test argument"),
            )
        })?;
        scope.check_active()?;
        count.set(count.get() + 1);
        Ok(Completion::Undefined)
    });
    let result = process.execution().in_realm(&realm, |scope| {
        let global = scope.global()?;
        let function = scope.function(&callback)?;
        scope.set(&global, PropertyKey::String(&units("read")), &function)?;
        let function = scope.function(&mark)?;
        scope.set(&global, PropertyKey::String(&units("mark")), &function)?;
        scope.evaluate(&units("try { read({get bubbles() {mark(1);return false}, get cancelable() {mark(2);return false}, get composed() {mark(3);return false}}) } catch (_) {};"))?;
        Ok(())
    });
    if short || value_cut {
        assert!(matches!(result, Err(Error::ConversionLedger(_))));
        assert_eq!(effects.get(), 0);
        if value_cut {
            assert_eq!(*getters.borrow(), [IntegerValue::Long(1)]);
            assert_eq!(ledger.usage(ledger.root())?.work(Work::Value), 1);
            assert_eq!(ledger.usage(ledger.root())?.work(Work::DictionaryMember), 1);
        } else {
            assert_eq!(
                *getters.borrow(),
                [IntegerValue::Long(1), IntegerValue::Long(2)]
            );
            assert_eq!(ledger.usage(ledger.root())?.work(Work::DictionaryMember), 2);
        }
    } else {
        result?;
        assert_eq!(effects.get(), 1);
        assert_eq!(
            *getters.borrow(),
            [
                IntegerValue::Long(1),
                IntegerValue::Long(2),
                IntegerValue::Long(3)
            ]
        );
        assert_eq!(ledger.usage(ledger.root())?.work(Work::Value), 4);
    }
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    Ok(())
}

fn profiling_preserves_composed_values(process: &mut Process) -> TestResult {
    use surgeist_ledger::EventFilter;
    let realm = process.execution().create_realm()?;
    for mode in [
        ProfileMode::Off,
        ProfileMode::Summary { timing: false },
        ProfileMode::Summary { timing: true },
        ProfileMode::Detailed {
            timing: true,
            event_limit: 8,
            filter: EventFilter::ScopeName("excluded"),
        },
    ] {
        let ledger = new_ledger(mode)?;
        let account = account(process, &realm, &ledger, Limits::unlimited())?;
        process.execution().in_realm(&realm, |scope| {
            let input = scope.evaluate(&units(
                "({bubbles: true, cancelable: false, composed: true})",
            ))?;
            let text = scope.string(&[0xd800, 120])?;
            let (dictionary, nullable, missing) = account.run(scope, |cx| {
                let dictionary = cx.value(&declarations::event_init(), &input)?;
                let nullable = cx.value(&types::DOM_STRING.nullable(), &text)?;
                let missing = cx.argument(
                    &declarations::dom_implementation_create_html_document_0_title(),
                    None,
                )?;
                Ok((dictionary, nullable, missing))
            })?;
            assert_eq!(
                *dictionary.get(),
                EventInit {
                    bubbles: true,
                    cancelable: false,
                    composed: true
                }
            );
            assert_eq!(
                nullable.get().as_ref().expect("text").as_code_units(),
                [0xd800, 120]
            );
            assert!(matches!(missing.get(), ArgumentValue::Missing));
            let usage = ledger.usage(ledger.root()).expect("ledger");
            assert_eq!(usage.work(Work::Value), 6);
            assert_eq!(usage.work(Work::CodeUnit), 2);
            assert_eq!(usage.work(Work::DictionaryMember), 3);
            assert_eq!(usage.work(Work::Argument), 1);
            assert_eq!(usage.live(Resource::BufferBytes), 4);
            Ok(())
        })?;
        assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
        assert_eq!(account.cleanup_status().failures(), Some(0));
    }
    Ok(())
}

fn nullable_admission(short_storage: bool) -> TestResult {
    let mut process = Process::bootstrap(surgeist_bindings_js::EngineKind::V8)?;
    let realm = process.execution().create_realm()?;
    let ledger = new_ledger(ProfileMode::Off)?;
    let limits = Limits::new(
        WorkTable {
            Value: 2,
            CodeUnit: 3,
            EnumerationLookup: 0,
            Argument: 0,
            DictionaryMember: 0,
            IteratorOperation: 0,
            SequenceItem: 0,
            SequenceMove: 0,
        },
        ResourceTable {
            BufferBytes: if short_storage { 5 } else { 6 },
        },
    );
    let account = account(&mut process, &realm, &ledger, limits)?;
    let result = process.execution().in_realm(&realm, |scope| {
        let input = scope.string(&units("abc"))?;
        account
            .run(scope, |cx| cx.value(&types::DOM_STRING.nullable(), &input))
            .map_err(Error::from)
    });
    if short_storage {
        assert!(matches!(result, Err(Error::ConversionLedger(_))));
        assert_eq!(ledger.usage(ledger.root())?.work(Work::CodeUnit), 0);
    } else {
        let value = result?;
        assert_eq!(
            value.get().as_ref().expect("text").as_code_units(),
            units("abc")
        );
        assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 6);
        drop(value);
    }
    assert_eq!(ledger.usage(ledger.root())?.live(Resource::BufferBytes), 0);
    Ok(())
}

pub(super) fn run(process: &mut Process) -> TestResult {
    dictionary_reads_and_reentry(process)?;
    nullable_and_declared_arguments(process)?;
    profiling_preserves_composed_values(process)?;
    Ok(())
}

pub(super) fn independent_owners() -> TestResult {
    dictionary_admission(false, false)?;
    dictionary_admission(true, false)?;
    dictionary_admission(false, true)?;
    nullable_admission(false)?;
    nullable_admission(true)
}
