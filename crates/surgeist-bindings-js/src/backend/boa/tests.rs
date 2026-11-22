use super::Engine;
use crate::{Error, backend::Engine as _};

pub(super) fn coerced_string_copy_is_exact_noncoercive_and_owner_checked() -> Result<(), Error> {
    let original = [0, 0xd800, 0x78, 0xdc00, 0xd83d, 0xde00];
    let retained = {
        let mut engine = Engine::bootstrap()?;
        let realm = engine.create_realm()?;
        let mut retained = None;
        engine.with_realm(&realm, &mut |entered| {
            let source = r#"
                globalThis.stringCoercions = 0;
                globalThis.stringInput = {
                    [Symbol.toPrimitive](hint) {
                        stringCoercions++;
                        if (hint !== 'string') throw 1;
                        return '\0\uD800x\uDC00\uD83D\uDE00';
                    }
                };
                stringInput
            "#;
            let input = entered.evaluate(&source.encode_utf16().collect::<Vec<_>>())?;
            let string = entered.coerce_string(&input)?;
            assert_eq!(string.len(), original.len());
            drop(input);
            entered.evaluate(
                &"stringInput[Symbol.toPrimitive] = () => { throw 2; }; stringInput = null"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )?;
            retained = Some(string);
            Ok(())
        })?;
        let string = retained.expect("successful entry retained the coerced string");
        engine.memory_pressure();
        engine.with_realm(&realm, &mut |entered| {
            // Copying the retained result must bypass the original object's
            // now-replaced coercion hook, even after collection.
            for _ in 0..2 {
                let mut output = [0xffff; 6];
                entered.copy_string(&string, &mut output)?;
                assert_eq!(output, original);
            }
            let calls = entered.evaluate(&"stringCoercions".encode_utf16().collect::<Vec<_>>())?;
            assert_eq!(&*entered.to_utf16(&calls)?, &[0x31]);
            let empty = entered.evaluate(&[0x27, 0x27])?;
            let empty = entered.coerce_string(&empty)?;
            assert_eq!(empty.len(), 0);
            entered.copy_string(&empty, &mut [])?;
            Ok(())
        })?;
        string
    };

    let mut engine = Engine::bootstrap()?;
    let realm = engine.create_realm()?;
    engine.with_realm(&realm, &mut |entered| {
        let mut output = [0xffff; 6];
        assert!(matches!(
            entered.copy_string(&retained, &mut output),
            Err(Error::ForeignOwner)
        ));
        assert_eq!(output, [0xffff; 6]);
        Ok(())
    })
}

#[test]
fn engine_scope_and_custody_contracts() -> Result<(), Box<dyn std::error::Error>> {
    // Keep the execution owner on this test thread across successive cases.
    assert!(crate::backend::validate_array_length(16_777_216).is_ok());
    assert!(matches!(
        crate::backend::validate_array_length(16_777_217),
        Err(Error::ArrayTooLong)
    ));
    coerced_string_copy_is_exact_noncoercive_and_owner_checked()?;
    crate::callback_object::retired_prototype_refuses_before_native_allocation()?;
    crate::conversion::tests::run()?;
    Ok(())
}
