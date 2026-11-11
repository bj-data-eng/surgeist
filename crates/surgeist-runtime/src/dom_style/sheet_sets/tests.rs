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
