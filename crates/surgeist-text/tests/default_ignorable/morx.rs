//! Complete AAT tables over the same retained licensed Ahem outline resource.
//! Safe byte mechanics follow the independently qualified maintained-provider
//! fixture; this module neither imports provider internals nor reads receipts.

use super::{A, Action, B, D, R, fixture, rewrite_tables, words};

#[derive(Clone, Copy, Debug)]
pub enum MorxAction {
    Delete,
    NoncontextualDelete,
    ReplaceThenDelete,
    Insert { marked: bool, delete_inserted: bool },
    Consume { delete_joint: bool },
    Skip,
}

fn longs(values: &[u32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|value| value.to_be_bytes())
        .collect()
}

fn long(value: usize) -> u32 {
    u32::try_from(value).unwrap()
}

fn morx(subtables: Vec<(u32, Vec<u8>)>) -> Vec<u8> {
    let count = long(subtables.len());
    let mut payload = Vec::new();
    for (kind, bytes) in subtables {
        let length = (12 + bytes.len() + 3) & !3;
        // Horizontal, logical-order subtable, enabled by chain default flag1.
        payload.extend(longs(&[long(length), 0x2000_0000 | kind, 1]));
        payload.extend(bytes);
        while !payload.len().is_multiple_of(4) {
            payload.push(0);
        }
    }
    let mut data = words(&[2, 0]);
    data.extend(longs(&[1, 1, long(16 + payload.len()), 0, count]));
    data.extend(payload);
    data
}

fn contextual(input: u16, output: u16) -> (u32, Vec<u8>) {
    let mut data = longs(&[5, 20, 28, 48, 64]);
    data.extend(words(&[8, input, 1, 4]));
    data.extend(words(&[0, 0, 0, 0, 1, 0, 0, 0, 0, 1]));
    data.extend(words(&[0, 0, 0xffff, 0xffff, 0, 0, 0xffff, 0]));
    data.extend(longs(&[4]));
    data.extend(words(&[8, input, 1, output]));
    (1, data)
}

fn insertion(marked: bool) -> (u32, Vec<u8>) {
    let mut data = longs(&[5, 20, 28, 48, 64]);
    data.extend(words(&[8, D, 1, 4]));
    data.extend(words(&[0, 0, 0, 0, 1, 0, 0, 0, 0, 1]));
    data.extend(words(&[0, 0, 0xffff, 0xffff]));
    if marked {
        // Initial mark is genuine input0: the lone DI is the selected anchor.
        data.extend(words(&[0, 0x0401, 0xffff, 0]));
    } else {
        data.extend(words(&[0, 0x0820, 0, 0xffff]));
    }
    data.extend(words(&[R]));
    (5, data)
}

fn ligature(second: u16) -> (u32, Vec<u8>) {
    // Two explicit component classes; unrelated members remain class1 and do
    // not enter the component stack merely because cluster custody is shared.
    let classes = words(&[6, 4, 2, 8, 1, 0, A, 4, second, 5]);
    let states = words(&[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 2, 0, 0, 1, 3]);
    let entries = words(&[0, 0, 0, 2, 0x8000, 0, 2, 0, 0, 0, 0xa000, 0]);
    let action_at = 28 + classes.len() + states.len() + entries.len();
    let component_at = action_at + 8;
    let ligature_at = component_at + (usize::from(D) + 1) * 2;
    let mut data = longs(&[
        6,
        28,
        long(28 + classes.len()),
        long(28 + classes.len() + states.len()),
        long(action_at),
        long(component_at),
        long(ligature_at),
    ]);
    data.extend(classes);
    data.extend(states);
    data.extend(entries);
    data.extend(longs(&[0, 0x8000_0000]));
    data.extend(vec![0; (usize::from(D) + 1) * 2]);
    data.extend(words(&[R]));
    (2, data)
}

fn positioning() -> Vec<u8> {
    // Real DFLT/kern PairPos: A35 before Q277 loses500 units (5px). That
    // positive control proves GPOS admission, without affecting A/DI/B cases.
    let mut data = words(&[1, 0, 10, 30, 44, 1]);
    data.extend(*b"DFLT");
    data.extend(words(&[8, 4, 0, 0, 0xffff, 1, 0, 1]));
    data.extend(*b"kern");
    data.extend(words(&[8, 0, 1, 0, 1, 4, 2, 0, 1, 8]));
    data.extend(words(&[
        1,
        18,
        4,
        0,
        1,
        12,
        1,
        super::Q,
        u16::from_be_bytes((-500_i16).to_be_bytes()),
        1,
        1,
        A,
    ]));
    data
}

pub fn morx_fixture(action: MorxAction, with_gpos: bool) -> Vec<u8> {
    let subtables = match action {
        MorxAction::Delete => vec![contextual(D, 0xffff)],
        MorxAction::NoncontextualDelete => vec![(4, words(&[8, D, 1, 0xffff]))],
        MorxAction::ReplaceThenDelete => vec![contextual(D, R), contextual(R, 0xffff)],
        MorxAction::Insert {
            marked,
            delete_inserted,
        } => {
            let mut tables = vec![insertion(marked)];
            if delete_inserted {
                tables.push(contextual(R, 0xffff));
            }
            tables
        }
        MorxAction::Consume { delete_joint } => {
            let mut tables = vec![ligature(D)];
            if delete_joint {
                tables.push(contextual(R, 0xffff));
            }
            tables
        }
        MorxAction::Skip => vec![ligature(B)],
    };
    let bytes = fixture(Some('\u{2065}'), Action::None);
    let mut tables = vec![(*b"morx", morx(subtables))];
    if with_gpos {
        tables.push((*b"GPOS", positioning()));
    }
    rewrite_tables(&bytes, tables)
}
