// Literal WOFF1 packaging of the retained 1.50 fixture. The directory uses
// original tag order and the payload uses original physical table order.
pub fn literal_woff1(sfnt: &[u8]) -> Vec<u8> {
    fn put32(bytes: &mut [u8], offset: usize, value: usize) {
        bytes[offset..offset + 4].copy_from_slice(&u32::try_from(value).unwrap().to_be_bytes());
    }

    let count = usize::from(u16_at(sfnt, 4));
    let mut wire = vec![0; 44 + 20 * count];
    wire[..4].copy_from_slice(b"wOFF");
    wire[4..8].copy_from_slice(&sfnt[..4]);
    wire[12..14].copy_from_slice(&sfnt[4..6]);
    let mut records = (0..count).collect::<Vec<_>>();
    records.sort_unstable_by_key(|&index| u32_at(sfnt, 20 + 16 * index));
    let mut decoded_len = 12 + 16 * count;
    for index in records {
        let record = 12 + 16 * index;
        let entry = 44 + 20 * index;
        let original = u32_at(sfnt, record + 8) as usize;
        let len = u32_at(sfnt, record + 12) as usize;
        let wire_offset = wire.len();
        wire[entry..entry + 4].copy_from_slice(&sfnt[record..record + 4]);
        put32(&mut wire, entry + 4, wire_offset);
        put32(&mut wire, entry + 8, len);
        put32(&mut wire, entry + 12, len);
        wire[entry + 16..entry + 20].copy_from_slice(&sfnt[record + 4..record + 8]);
        wire.extend_from_slice(&sfnt[original..original + len]);
        wire.resize(wire.len().next_multiple_of(4), 0);
        decoded_len += len.next_multiple_of(4);
    }
    let wire_len = wire.len();
    put32(&mut wire, 8, wire_len);
    put32(&mut wire, 16, decoded_len);
    wire
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap())
}
