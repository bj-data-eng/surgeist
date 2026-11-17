//! A dimension table holds one value per dimension and reaches each value by
//! the dimension that names it.

use surgeist_ledger::Dimension;

surgeist_ledger::dimensions! {
    enum Work => WorkTable, WorkFields { Read, Visit, Emit, Copy, Compare }
}

surgeist_ledger::dimensions! {
    enum Single => SingleTable, SingleFields { Only }
}

const NAMED: WorkTable<u64> = WorkTable::new(WorkFields {
    Read: 11,
    Visit: 22,
    Emit: 33,
    Copy: 44,
    Compare: 55,
});

#[test]
fn a_table_holds_the_value_its_fields_named_for_each_dimension() {
    assert_eq!(NAMED[Work::Read], 11);
    assert_eq!(NAMED[Work::Visit], 22);
    assert_eq!(NAMED[Work::Emit], 33);
    assert_eq!(NAMED[Work::Copy], 44);
    assert_eq!(NAMED[Work::Compare], 55);
    for (position, &dimension) in Work::ALL.iter().enumerate() {
        let expected = 11 * (u64::try_from(position).unwrap() + 1);
        assert_eq!(*Work::get(&NAMED, dimension), expected);
    }
}

#[test]
fn a_table_built_by_a_function_holds_its_value_for_each_dimension() {
    let mut calls = Vec::new();
    let table = Work::from_fn(|dimension| {
        calls.push(dimension);
        match dimension {
            Work::Read => 11_u64,
            Work::Visit => 22,
            Work::Emit => 33,
            Work::Copy => 44,
            Work::Compare => 55,
        }
    });
    assert_eq!(table, NAMED);
    assert_eq!(calls, Work::ALL);
}

#[test]
fn a_splat_table_holds_one_value_for_every_dimension() {
    const SPLAT: WorkTable<u8> = WorkTable::splat(7);
    for &dimension in Work::ALL {
        assert_eq!(SPLAT[dimension], 7);
    }
}

#[test]
fn fields_named_over_a_splat_keep_the_splat_for_the_rest() {
    let table = WorkTable::new(WorkFields {
        Emit: 3_u64,
        ..WorkFields::splat(9)
    });
    for &dimension in Work::ALL {
        let expected = if dimension == Work::Emit { 3 } else { 9 };
        assert_eq!(table[dimension], expected);
    }
}

#[test]
fn an_update_reaches_only_the_dimension_it_names() {
    for &updated in Work::ALL {
        let mut by_index = NAMED;
        by_index[updated] = 1000;
        let mut by_trait = NAMED;
        *Work::get_mut(&mut by_trait, updated) = 1000;
        assert_eq!(by_index, by_trait);
        for &dimension in Work::ALL {
            let expected = if dimension == updated {
                1000
            } else {
                NAMED[dimension]
            };
            assert_eq!(by_index[dimension], expected);
        }
    }
}

#[test]
fn a_table_of_one_dimension_holds_its_value() {
    let mut table = SingleTable::new(SingleFields { Only: 5_u32 });
    assert_eq!(table[Single::Only], 5);
    table[Single::Only] += 1;
    assert_eq!(*Single::get(&table, Single::Only), 6);
}

#[test]
fn a_table_formats_each_value_under_the_name_of_its_dimension() {
    assert_eq!(
        format!("{NAMED:?}"),
        "WorkTable { Read: 11, Visit: 22, Emit: 33, Copy: 44, Compare: 55 }"
    );
}
