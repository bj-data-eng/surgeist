use super::*;
use crate::conversion::{Argument, ConversionAccount, Converted, Resource, types};
use crate::{Arguments, Callback, Completion, Error, ObjectPrototype, Process, PropertyKey};
use surgeist_ledger::{Limits, Operation, ProfileMode, Tariffs};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

#[derive(Clone, Copy)]
enum Case {
    Brand,
    Callable,
    Sequence,
    Dictionary,
    Boolean,
    Number,
    String,
}

struct Brand;

const CASES: &[Candidate<Case>] = &[
    Candidate::new(Case::Dictionary, &[Category::Dictionary], false, false),
    Candidate::new(Case::Number, &[Category::Number], false, false),
    Candidate::new(Case::String, &[Category::String], false, false),
    Candidate::new(Case::Callable, &[Category::Callable], false, false),
    Candidate::new(Case::Sequence, &[Category::Sequence], false, false),
    Candidate::new(Case::Boolean, &[Category::Boolean], false, false),
    Candidate::new(Case::Brand, &[Category::Interface("Fixture")], false, false),
];

fn dispatch<'s>(
    cx: &mut Conversion<'_, 's>,
    args: &Arguments<'s>,
) -> Result<&'static str, Failure> {
    cx.effective_arity("fixture", args.len(), 2, &[2])?;
    let prefix = Argument::new("fixture/prefix", 0, false, types::DOM_STRING, None);
    let _prefix = cx.generated_required_argument(&prefix, args.get(0))?;
    let input = args
        .get(1)
        .ok_or_else(|| cx.validation(Rule::Overload("fixture")))?;
    cx.work(Work::Argument, 1).map_err(Failure)?;
    let (case, selected) = cx.select(
        SelectionKind::Overload,
        "fixture",
        input,
        CASES,
        |cx, name, value| {
            assert_eq!(name, "Fixture");
            let result = cx.scope.native_object_token::<Brand>(value);
            cx.core(result).map(|token| token.is_some())
        },
    )?;
    Ok(match case {
        Case::Brand => "brand",
        Case::Callable => "callable",
        Case::Sequence => {
            let words = cx.selected_value(&types::DOM_STRING.sequence(), selected)?;
            let words = words
                .get()
                .iter()
                .map(|word| word.get().as_code_units())
                .collect::<Vec<_>>();
            assert_eq!(words, [&[97][..], &[98][..]]);
            "sequence"
        }
        Case::Dictionary => {
            cx.selected_value(
                &crate::conversion::declarations::element_creation_options(),
                selected,
            )?;
            "dictionary"
        }
        Case::Boolean => {
            cx.selected_value(&types::BOOLEAN, selected)?;
            "boolean"
        }
        Case::Number => {
            cx.selected_value(&types::DOUBLE, selected)?;
            "number"
        }
        Case::String => {
            cx.selected_value(&types::DOM_STRING, selected)?;
            "string"
        }
    })
}

fn observable_paths() -> TestResult {
    let mut process = Process::bootstrap()?;
    let origin = process.execution().create_realm()?;
    let brand = process.execution().in_realm(&origin, |scope| {
        let brand = scope
            .project_native_object(ObjectPrototype::Default, Brand, |_| Ok::<(), Error>(()))?;
        Ok(scope.retain(&brand))
    })?;
    let realm = process.execution().create_realm()?;
    let ledger = Operation::with_profile("test", Limits::unlimited(), 3, ProfileMode::Off)?.share();
    process.execution().in_realm(&realm, |scope| {
        let account = ConversionAccount::new(scope, ledger.clone(), ledger.root(), Limits::unlimited(), Tariffs::identity())?;
        let fixture = Callback::new(move |scope, args| {
            let name = account.run(scope, |cx| dispatch(cx, &args))?;
            Ok(Completion::Return(scope.string(&units(name))?))
        });
        let global = scope.global()?;
        let function = scope.function(&fixture)?;
        scope.set(&global, PropertyKey::String(&units("fixture")), &function)?;
        let brand = scope.import(&brand)?;
        scope.set(&global, PropertyKey::String(&units("brand")), &brand)?;
        let result = scope.evaluate(&units(r#"(() => {
            const trace = [];
            const sentinel = {};
            const prefix = {toString() { trace.push('prefix'); return 'p'; }};
            const iterable = {
                get [Symbol.iterator]() {
                    trace.push('iterator');
                    if (fixture('inner', true) !== 'boolean') throw 'reentry';
                    Object.defineProperty(this, Symbol.iterator, {get() {throw 'second lookup';}});
                    return function() {
                        if (this !== iterable) throw 'wrong receiver';
                        trace.push('method');
                        return ['a', 'b'][Symbol.iterator]();
                    };
                }
            };
            if (fixture(prefix, iterable) !== 'sequence' || trace.join() !== 'prefix,iterator,method')
                return 'iterator order';
            Object.defineProperty(brand, Symbol.iterator, {get() { throw 'unused iterator'; }});
            if (fixture('p', brand) !== 'brand' ||
                fixture('p', Object.create(Object.getPrototypeOf(brand))) !== 'dictionary')
                return 'brand and callable precedence';
            const callable = () => {};
            Object.defineProperty(callable, Symbol.iterator, {get() { throw 'unused iterator'; }});
            if (fixture('p', callable) !== 'callable') return 'callable before iterable';
            trace.length = 0;
            const options = {
                get [Symbol.iterator]() { trace.push('absent iterator'); return null; },
                get is() { trace.push('dictionary member'); return 'x'; }
            };
            if (fixture(prefix, options) !== 'dictionary' ||
                trace.join() !== 'prefix,absent iterator,dictionary member')
                return 'dictionary fallback order';
            if (fixture('p', 1) !== 'number' || fixture('p', false) !== 'boolean' ||
                fixture('p', 'x') !== 'string' || fixture('p', null) !== 'dictionary')
                return 'primitive and nullish choices';
            try { fixture(prefix, {get [Symbol.iterator]() {throw sentinel;}}); return 'missing throw'; }
            catch (error) { if (error !== sentinel) return 'throw identity'; }
            try { fixture(prefix, {[Symbol.iterator]: 1}); return 'noncallable iterator'; }
            catch (error) { if (!(error instanceof TypeError)) return 'iterator failure type'; }
            trace.length = 0;
            try { fixture(prefix); return 'accepted missing argument'; }
            catch (error) { if (!(error instanceof TypeError) || trace.length) return 'arity before coercion'; }
            return 'ok';
        })()"#))?;
        assert_eq!(scope.to_utf16(&result)?, units("ok"));
        Ok(())
    })?;
    Ok(())
}

fn category_precedence() -> TestResult {
    let mut process = Process::bootstrap()?;
    let realm = process.execution().create_realm()?;
    let ledger = Operation::with_profile("test", Limits::unlimited(), 3, ProfileMode::Off)?.share();
    process.execution().in_realm(&realm, |scope| {
        let account = ConversionAccount::new(
            scope,
            ledger.clone(),
            ledger.root(),
            Limits::unlimited(),
            Tariffs::identity(),
        )?;
        let optional = [
            Candidate::new("dictionary", &[Category::Dictionary], false, false),
            Candidate::new("optional string", &[Category::String], true, false),
        ];
        let nullable = [
            Candidate::new("number", &[Category::Number], false, false),
            Candidate::new("nullable string", &[Category::String], false, true),
        ];
        let object = [
            Candidate::new("string", &[Category::String], false, false),
            Candidate::new("object", &[Category::Object], false, false),
        ];
        let undefined = [
            Candidate::new("number", &[Category::Number], false, true),
            Candidate::new("undefined", &[Category::Undefined], false, false),
        ];
        for (kind, candidates, word, expected) in [
            (
                SelectionKind::Overload,
                &optional,
                "undefined",
                "optional string",
            ),
            (SelectionKind::Overload, &optional, "null", "dictionary"),
            (
                SelectionKind::Overload,
                &nullable,
                "null",
                "nullable string",
            ),
            (
                SelectionKind::Overload,
                &object,
                "({ get [Symbol.iterator]() { throw 'no lookup'; } })",
                "object",
            ),
            (SelectionKind::Union, &undefined, "undefined", "undefined"),
            (SelectionKind::Union, &undefined, "null", "number"),
        ] {
            let value = scope.evaluate(&units(word))?;
            let actual = account.run(scope, |cx| {
                let (tag, _) =
                    cx.select(kind, "fixture", &value, candidates, |_, _, _| Ok(false))?;
                Ok(tag)
            })?;
            assert_eq!(actual, expected);
        }
        Ok(())
    })?;
    Ok(())
}

fn selection_cutoff(limit: u64) -> TestResult {
    let mut process = Process::bootstrap()?;
    let realm = process.execution().create_realm()?;
    let ledger = Operation::with_profile("test", Limits::unlimited(), 3, ProfileMode::Off)?.share();
    let limits = Limits::from_fn(
        |work| {
            if work == Work::Selection {
                limit
            } else {
                u64::MAX
            }
        },
        |_| u64::MAX,
    );
    let outcome = process.execution().in_realm(&realm, |scope| {
        let account = ConversionAccount::new(
            scope,
            ledger.clone(),
            ledger.root(),
            limits,
            Tariffs::identity(),
        )?;
        let input = scope.evaluate(&units("['a', 'b']"))?;
        let output: Converted<Vec<Converted<crate::DomString>>> = account.run(scope, |cx| {
            let (_, selected) = cx.select(
                SelectionKind::Overload,
                "fixture",
                &input,
                CASES,
                |_, _, _| Ok(false),
            )?;
            cx.selected_value(&types::DOM_STRING.sequence(), selected)
        })?;
        assert_eq!(output.get().len(), 2);
        assert!(
            ledger
                .usage(ledger.root())
                .unwrap()
                .live(Resource::BufferBytes)
                > 0
        );
        Ok(())
    });
    let used = ledger.usage(ledger.root())?;
    assert_eq!(used.live(Resource::BufferBytes), 0);
    assert_eq!(used.work(Work::Selection), limit);
    if limit == 2 {
        outcome?;
        assert_eq!(used.work(Work::IteratorOperation), 11);
    } else {
        assert!(matches!(outcome, Err(Error::ConversionLedger(_))));
        assert_eq!(used.work(Work::IteratorOperation), 0);
    }
    Ok(())
}

pub(in crate::conversion) fn run() -> TestResult {
    observable_paths()?;
    category_precedence()?;
    selection_cutoff(2)?;
    selection_cutoff(1)?;
    Ok(())
}
