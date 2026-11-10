use super::{TextRangeError, range_fields};

const U32_LIMIT: usize = u32::MAX as usize;

#[test]
fn a_range_over_the_whole_block_has_no_fields() {
    assert_eq!(range_fields(0, 0, 0), Ok(None));
    assert_eq!(range_fields(0, 7, 7), Ok(None));
}

#[test]
fn a_part_of_the_block_keeps_its_start_and_length() {
    assert_eq!(range_fields(0, 0, 5), Ok(Some((0, 0))));
    assert_eq!(range_fields(2, 5, 7), Ok(Some((2, 3))));
    assert_eq!(range_fields(7, 7, 7), Ok(Some((7, 0))));
}

#[test]
fn reversed_and_overlong_ranges_are_refused() {
    assert_eq!(
        range_fields(5, 3, 10),
        Err(TextRangeError::Reversed { start: 5, end: 3 })
    );
    assert_eq!(
        range_fields(0, 11, 10),
        Err(TextRangeError::OutOfBounds {
            end: 11,
            block_len: 10
        })
    );
    assert_eq!(
        range_fields(usize::MAX, usize::MAX, 10),
        Err(TextRangeError::OutOfBounds {
            end: usize::MAX,
            block_len: 10
        })
    );
}

#[test]
fn the_longest_part_has_one_unit_less_than_the_whole_block_marker() {
    assert_eq!(
        range_fields(1, U32_LIMIT, U32_LIMIT),
        Ok(Some((1, u32::MAX - 1)))
    );
    assert_eq!(super::TextBlock::MAX_RANGE_UNITS, U32_LIMIT - 1);
}

#[cfg(target_pointer_width = "64")]
#[test]
fn a_part_as_long_as_the_whole_block_marker_is_unrepresentable() {
    assert_eq!(
        range_fields(1, U32_LIMIT + 1, U32_LIMIT + 2),
        Err(TextRangeError::Unrepresentable {
            start: 1,
            len: U32_LIMIT
        })
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn a_part_starting_past_u32_is_unrepresentable() {
    let start = U32_LIMIT + 1;
    assert_eq!(
        range_fields(start, start + 2, start + 3),
        Err(TextRangeError::Unrepresentable { start, len: 2 })
    );
    assert_eq!(
        range_fields(U32_LIMIT, U32_LIMIT, U32_LIMIT + 1),
        Ok(Some((u32::MAX, 0)))
    );
}

#[cfg(target_pointer_width = "64")]
#[test]
fn a_whole_block_longer_than_u32_has_no_fields() {
    assert_eq!(range_fields(0, U32_LIMIT + 5, U32_LIMIT + 5), Ok(None));
}
