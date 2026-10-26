//! Finite Irish prefix facts from the source-qualified examples in Text's
//! casing contract. This is Text-owned execution, not copied browser code.
//! Matches inspect immutable input at a real word start; styles remain owned
//! by each affected component when these facts are consumed.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct Prefix {
    /// Initial components that remain lowercase in the uppercase operation.
    pub(super) preserve: usize,
    /// Exact removed original hyphen component, relative to the word start.
    pub(super) remove: Option<usize>,
    /// Exact vowel component receiving a hyphen before its lowercase mapping.
    pub(super) insert_before: Option<usize>,
}

fn vowel(cp: u32) -> bool {
    matches!(
        cp,
        0x61 | 0xe1 | 0x65 | 0xe9 | 0x69 | 0xed | 0x6f | 0xf3 | 0x75 | 0xfa
    )
}

fn capital_vowel(cp: u32) -> bool {
    matches!(
        cp,
        0x41 | 0xc1 | 0x45 | 0xc9 | 0x49 | 0xcd | 0x4f | 0xd3 | 0x55 | 0xda
    )
}

pub(super) fn prefix(input: &[u32]) -> Prefix {
    let first = input.first().copied();
    let second = input.get(1).copied();
    let third = input.get(2).copied();
    let mut result = Prefix::default();
    if matches!(
        (first, second, third),
        (Some(0x62), Some(0x68), Some(0x66 | 0x46))
    ) {
        result.preserve = 2;
    } else if matches!(
        (first, second),
        (Some(0x62), Some(0x70 | 0x50))
            | (Some(0x64), Some(0x74 | 0x54))
            | (Some(0x67), Some(0x63 | 0x43))
            | (Some(0x6d), Some(0x62 | 0x42))
            | (Some(0x6e), Some(0x64 | 0x44 | 0x67 | 0x47))
    ) || first == Some(0x68) && second.is_some_and(capital_vowel)
    {
        result.preserve = 1;
    } else if matches!(first, Some(0x6e | 0x74)) {
        if second.is_some_and(capital_vowel) {
            result.preserve = 1;
            result.insert_before = Some(1);
        } else if matches!(second, Some(0x2d | 0x2010 | 0x2011)) && third.is_some_and(vowel) {
            result.preserve = 1;
            result.remove = Some(1);
        }
    }
    if first == Some(0x74)
        && matches!(second, Some(0x73 | 0x53))
        && third.is_some_and(|cp| {
            vowel(cp) || capital_vowel(cp) || matches!(cp, 0x6c | 0x4c | 0x6e | 0x4e | 0x72 | 0x52)
        })
    {
        result.preserve = 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::casing::{Locale, Mapper, Operation};

    fn mapped(input: &str, operation: Operation) -> (String, Vec<Vec<u32>>) {
        let input: Vec<_> = input.chars().map(u32::from).collect();
        let facts = prefix(&input);
        let mapper = Mapper::new().unwrap();
        let rows: Vec<_> = (0..input.len())
            .map(|at| {
                if operation == Operation::Upper && at < facts.preserve {
                    return vec![input[at]];
                }
                if operation == Operation::Upper && Some(at) == facts.remove {
                    return vec![];
                }
                let mut result = mapper.map(&input, at, operation, Locale::Irish).unwrap();
                if operation == Operation::Lower && Some(at) == facts.insert_before {
                    result.insert(0, 0x2d);
                }
                result
            })
            .collect();
        let output = rows
            .iter()
            .flatten()
            .map(|&cp| char::from_u32(cp).unwrap())
            .collect();
        (output, rows)
    }

    #[test]
    fn finite_prefix_facts_retain_exact_original_component_ownership() {
        for (input, expected) in [
            ("hataí", "HATAÍ"),
            ("hÁite", "hÁITE"),
            ("nAthair", "nATHAIR"),
            ("Nathair", "NATHAIR"),
            ("t-uisce", "tUISCE"),
            ("tslí", "tSLÍ"),
            ("bhfad", "bhFAD"),
            ("T-léine", "T-LÉINE"),
            ("scríobhfaidh", "SCRÍOBHFAIDH"),
            ("n\u{2010}óg", "nÓG"),
            ("t\u{2011}athair", "tATHAIR"),
        ] {
            assert_eq!(mapped(input, Operation::Upper).0, expected, "{input}");
        }
        assert_eq!(mapped("nÓg", Operation::Lower).0, "n-óg");
        assert_eq!(mapped("NÓG", Operation::Lower).0, "nóg");
        let (_, rows) = mapped("t-uisce", Operation::Upper);
        assert_eq!(rows[0], [0x74]);
        assert!(rows[1].is_empty());
        assert_eq!(rows[2], [0x55]);
        let (_, rows) = mapped("nÓg", Operation::Lower);
        assert_eq!(rows[0], [0x6e]);
        assert_eq!(rows[1], [0x2d, 0xf3]);
    }
}
