use super::*;
use surgeist_html::values::TextBlock;

fn same(left: &[u16], right: &[u16]) -> Result<bool, DomStyleRuntimeError> {
    Ok(left == right)
}

fn exact(units: &[u16]) -> DocumentString {
    let mut backing = Vec::new();
    backing.try_reserve_exact(units.len()).unwrap();
    backing.extend_from_slice(units);
    assert_eq!(backing.capacity(), units.len());
    DocumentString::from_utf16(backing)
}

/// The state of a document whose preferred and last selected names are `preferred` and `selected`.
fn state(preferred: &DocumentString, selected: &DocumentString) -> State {
    let RootSets::Document(initial) = RootSets::new(true) else {
        unreachable!("a Document root")
    };
    let (next, _) = initial
        .transition(preferred, Action::Preferred, same)
        .unwrap();
    let (next, _) = next.transition(selected, Action::Select, same).unwrap();
    next
}

#[test]
fn names_that_borrow_one_block_are_charged_by_length_and_the_block_once() {
    let owners = state(&exact(&[97, 98, 99, 100]), &exact(&[101, 102, 103, 104]));
    let mut units = Vec::with_capacity(512);
    units.extend([97_u16, 98, 99, 100, 101, 102, 103, 104]);
    let block = TextBlock::from_utf16(units);
    let borrowers = state(&block.string(0..4).unwrap(), &block.string(4..8).unwrap());
    let block_bytes = size_of::<Vec<u16>>() + block.capacity() * size_of::<u16>();
    assert_eq!(
        borrowers.additional_backing(None).unwrap(),
        owners.additional_backing(None).unwrap() + block_bytes
    );
}

#[test]
fn a_borrower_name_accounts_by_length_and_its_block_apart_and_an_owner_by_capacity() {
    let mut units = Vec::with_capacity(512);
    units.extend([97_u16, 98, 99, 100, 101, 102, 103, 104]);
    let block = TextBlock::from_utf16(units);
    let borrower = block.string(0..4).unwrap();
    assert_eq!(
        occurrence_backing(&borrower).unwrap(),
        (size_of::<Vec<u16>>() + 4 * size_of::<u16>(), 4)
    );
    assert_eq!(
        block_backing(&borrower),
        Some((block.retained_bytes(), block.capacity()))
    );
    let owner = DocumentString::from_utf16(Vec::with_capacity(64));
    assert_eq!(
        occurrence_backing(&owner).unwrap(),
        (
            size_of::<Vec<u16>>() + owner.retained_units() * size_of::<u16>(),
            owner.retained_units()
        )
    );
    assert_eq!(block_backing(&owner), None);
}

#[test]
fn names_of_two_blocks_are_charged_one_block_each_and_a_shared_alias_none() {
    let make = || {
        let mut units = Vec::with_capacity(512);
        units.extend([97_u16, 98, 99, 100]);
        TextBlock::from_utf16(units)
    };
    let (first, second) = (make(), make());
    let state = state(&first.string(0..2).unwrap(), &second.string(0..2).unwrap());
    let occurrence = 2 * (size_of::<Vec<u16>>() + 2 * size_of::<u16>());
    assert_eq!(
        state.additional_backing(None).unwrap(),
        occurrence + first.retained_bytes() + second.retained_bytes()
    );
    // A previous state that borrows the first block already carries its charge.
    let previous = self::state(&first.string(2..4).unwrap(), &second.string(2..4).unwrap());
    assert_eq!(
        state.additional_backing(Some(&previous)).unwrap(),
        occurrence
    );
}
