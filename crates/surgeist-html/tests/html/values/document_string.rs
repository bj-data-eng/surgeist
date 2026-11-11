//! The document string handle: content comparison across blocks and ranges, range bounds,
//! storage identity, retained capacity and in-place construction.
use std::{collections::hash_map::RandomState, hash::BuildHasher, sync::Arc};
use surgeist_html::values::{DocumentString, TextBlock, TextRangeError};

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

#[cfg(target_pointer_width = "64")]
#[test]
fn handle_is_sixteen_bytes_and_keeps_its_niche() {
    assert_eq!(size_of::<DocumentString>(), 16);
    assert_eq!(size_of::<Option<DocumentString>>(), 16);
}

#[test]
fn equal_content_in_different_blocks_and_ranges_is_equal_with_one_hash() {
    let authored = DocumentString::from("abc");
    let early = TextBlock::from_utf16(units("abcxyz")).string(0..3).unwrap();
    let late = TextBlock::from_utf16(units("xxabc")).string(2..5).unwrap();
    let middle = TextBlock::from_utf16(units("..abc.."))
        .string(2..5)
        .unwrap();
    let state = RandomState::new();
    // The hash of the units as a slice is the independent oracle.
    let expected = state.hash_one(&[97_u16, 98, 99][..]);
    for string in [&authored, &early, &late, &middle] {
        assert_eq!(string.code_units(), [97, 98, 99]);
        assert_eq!(state.hash_one(string), expected);
        for other in [&authored, &early, &late, &middle] {
            assert_eq!(string, other);
        }
    }
}

#[test]
fn different_content_is_unequal_in_every_block_arrangement() {
    let block = TextBlock::from_utf16(units("abcabd"));
    let first = block.string(0..3).unwrap();
    let second = block.string(3..6).unwrap();
    let prefix = block.string(0..2).unwrap();
    assert_ne!(first, second);
    assert_ne!(first, prefix);
    assert_ne!(first, DocumentString::from("abd"));
    assert_ne!(first, DocumentString::default());
}

#[test]
fn isolated_surrogates_and_nulls_survive_a_range() {
    let block = TextBlock::from_utf16(vec![0x61, 0xd800, 0, 0xdfff, 0x62]);
    let range = block.string(1..4).unwrap();
    assert_eq!(range.code_units(), [0xd800, 0, 0xdfff]);
    assert_eq!(range.code_points().collect::<Vec<_>>(), [0xd800, 0, 0xdfff]);
    assert_eq!(range.to_utf8().unwrap_err().code_unit_offset(), 0);
}

#[test]
fn empty_strings_are_equal_whatever_their_origin() {
    let block = TextBlock::from_utf16(units("abc"));
    let empties = [
        DocumentString::default(),
        DocumentString::from(""),
        DocumentString::from_utf16(Vec::new()),
        TextBlock::from_utf16(Vec::new()).whole(),
        block.string(0..0).unwrap(),
        block.string(1..1).unwrap(),
        block.string(3..3).unwrap(),
    ];
    let state = RandomState::new();
    let hash = state.hash_one(&empties[0]);
    for empty in &empties {
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(empty.code_units(), []);
        assert_eq!(empty, &empties[0]);
        assert_eq!(state.hash_one(empty), hash);
    }
}

#[test]
fn range_bounds_are_rejected_without_a_panic() {
    let block = TextBlock::from_utf16(units("abcd"));
    let (start, end) = (3, 1);
    let reversed = block.string(start..end);
    assert_eq!(
        reversed.unwrap_err(),
        TextRangeError::Reversed { start: 3, end: 1 }
    );
    assert_eq!(
        block.string(2..5).unwrap_err(),
        TextRangeError::OutOfBounds {
            end: 5,
            block_len: 4
        }
    );
    assert_eq!(
        block.string(0..usize::MAX).unwrap_err(),
        TextRangeError::OutOfBounds {
            end: usize::MAX,
            block_len: 4
        }
    );
    assert_eq!(
        block.string(usize::MAX..usize::MAX).unwrap_err(),
        TextRangeError::OutOfBounds {
            end: usize::MAX,
            block_len: 4
        }
    );
    assert!(block.get(start..end).is_none());
    assert!(block.get(2..5).is_none());
    assert_eq!(block.get(1..3), Some(&[98_u16, 99][..]));
    assert!(!block.string(4..4).unwrap().is_whole_block());
    assert!(block.string(0..4).unwrap().is_whole_block());
    assert_eq!(block.string(1..4).unwrap().range(), 1..4);
    assert!(
        TextRangeError::Reversed { start: 3, end: 1 }
            .to_string()
            .contains("3..1")
    );
}

#[test]
fn a_whole_range_is_the_whole_block_string() {
    let block = TextBlock::from_utf16(units("abcd"));
    let whole = block.string(0..4).unwrap();
    assert!(whole.is_whole_block());
    assert!(whole.shares_storage_with(&block.whole()));
    assert_eq!(whole.range(), 0..4);
}

#[test]
fn storage_identity_is_the_same_range_of_the_same_block() {
    let block = TextBlock::from_utf16(units("abcabc"));
    let first = block.string(0..3).unwrap();
    let again = block.string(0..3).unwrap();
    let second = block.string(3..6).unwrap();
    let other_block = TextBlock::from_utf16(units("abcabc")).string(0..3).unwrap();
    assert!(first.shares_storage_with(&first.clone()));
    assert!(first.shares_storage_with(&again));
    assert!(!first.shares_storage_with(&second));
    assert!(!first.shares_storage_with(&other_block));
    assert_eq!(first, other_block);
    assert!(block.shares_allocation_with(first.block()));
    assert!(!block.shares_allocation_with(other_block.block()));
    assert_eq!(first.block().address(), second.block().address());
    assert_ne!(first.block().address(), other_block.block().address());
}

#[test]
fn authored_strings_own_one_block_each() {
    let first = DocumentString::from("same");
    let second = DocumentString::from("same");
    assert_eq!(first, second);
    assert!(!first.shares_storage_with(&second));
    assert!(first.is_whole_block());
    assert_eq!(first.block().holders(), 1);
    let alias = first.clone();
    assert_eq!(first.block().holders(), 2);
    assert!(first.shares_storage_with(&alias));
}

#[test]
fn retained_units_count_the_whole_block_capacity() {
    let mut vector = Vec::with_capacity(31);
    vector.extend(units("abcdef"));
    let capacity = vector.capacity();
    assert!(capacity >= 31);
    let whole = DocumentString::from_utf16(vector);
    assert_eq!(whole.retained_units(), capacity);
    let range = whole.block().string(1..3).unwrap();
    assert_eq!(range.len(), 2);
    assert_eq!(range.retained_units(), capacity);
    assert_eq!(DocumentString::default().retained_units(), 0);
}

#[test]
fn only_a_sole_whole_block_string_changes_in_place() {
    let mut vector = Vec::with_capacity(16);
    vector.extend(units("ab"));
    let mut value = DocumentString::from_utf16(vector);
    let capacity = value.retained_units();
    value.unique_units_mut().unwrap().extend(units("cd"));
    assert_eq!(value.code_units(), [97, 98, 99, 100]);
    assert_eq!(value.len(), 4);
    assert_eq!(value.retained_units(), capacity);
    value.unique_units_mut().unwrap().truncate(1);
    assert_eq!(value.code_units(), [97]);
    assert_eq!(value, DocumentString::from("a"));

    let alias = value.clone();
    assert!(value.unique_units_mut().is_none());
    drop(alias);
    assert!(value.unique_units_mut().is_some());

    let weak = value.block().downgrade();
    assert!(value.unique_units_mut().is_none());
    drop(weak);
    assert!(value.unique_units_mut().is_some());

    let mut range = value.block().string(0..0).unwrap();
    assert!(range.unique_units_mut().is_none());
    let mut shared = TextBlock::from_utf16(units("abc")).string(0..2).unwrap();
    assert!(shared.unique_units_mut().is_none());
}

#[test]
fn weak_blocks_follow_the_strong_holders() {
    let value = DocumentString::from("abc");
    let weak = value.block().downgrade();
    assert_eq!(value.block().weak_holders(), 1);
    assert_eq!(weak.holders(), 1);
    assert_eq!(weak.address(), value.block().address());
    assert_eq!(weak.upgrade().unwrap().units(), [97, 98, 99]);
    drop(value);
    assert_eq!(weak.holders(), 0);
    assert!(weak.upgrade().is_none());
}

#[test]
fn a_whole_block_hands_its_allocation_to_shared_interfaces_without_copying() {
    let mut vector = Vec::with_capacity(40);
    vector.extend(units("abcdef"));
    let backing = Arc::new(vector);
    let value = DocumentString::from_shared_utf16(Arc::clone(&backing));
    assert!(value.is_whole_block());
    assert_eq!(value.retained_units(), backing.capacity());
    let handed = value.to_shared_utf16();
    assert!(Arc::ptr_eq(&handed, &backing));
    assert_eq!(Arc::strong_count(&backing), 3);
    assert_eq!(handed.as_slice(), value.code_units());
}

#[test]
fn a_partial_range_copies_its_units_for_shared_interfaces() {
    let value = DocumentString::from("abcdef");
    let range = value.block().string(1..4).unwrap();
    let copy = range.to_shared_utf16();
    assert_eq!(copy.as_slice(), [98, 99, 100]);
    assert!(!Arc::ptr_eq(&copy, &value.to_shared_utf16()));
    assert_eq!(value.block().holders(), 2);
    assert_eq!(Arc::strong_count(&copy), 1);
}

#[test]
fn debug_names_the_code_units() {
    assert_eq!(
        format!("{:?}", DocumentString::from("a")),
        "DocumentString { code_units: [97] }"
    );
}

#[test]
fn a_range_over_a_whole_block_is_still_a_borrower_of_it() {
    let block = TextBlock::from_utf16(units("abcd"));
    let full = block.string(0..4).unwrap();
    assert!(!full.is_whole_block());
    assert_eq!(full.range(), 0..4);
    assert!(block.whole().is_whole_block());
}

#[test]
fn a_range_counts_its_own_length_and_a_whole_block_string_its_capacity() {
    let mut vector = Vec::with_capacity(31);
    vector.extend(units("abcdef"));
    let capacity = vector.capacity();
    let owner = DocumentString::from_utf16(vector);
    assert_eq!(owner.retained_units(), capacity);
    let block = owner.block();
    for range in [0..6, 1..3, 4..4, 6..6] {
        let borrower = block.string(range.clone()).unwrap();
        assert_eq!(borrower.retained_units(), range.len());
    }
}
