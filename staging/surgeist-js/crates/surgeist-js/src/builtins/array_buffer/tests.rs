use crate::object::JsArrayBuffer;
use crate::{TestAction, run_test_actions};

#[test]
fn create_byte_data_block() {
    run_test_actions([TestAction::inspect_context(|context| {
        // Sunny day
        assert!(super::create_byte_data_block(100, None, context).is_ok());

        // Rainy day
        assert!(super::create_byte_data_block(u64::MAX, None, context).is_err());
    })]);
}

#[test]
fn create_shared_byte_data_block() {
    run_test_actions([TestAction::inspect_context(|context| {
        // Sunny day
        assert!(super::shared::create_shared_byte_data_block(100, context).is_ok());

        // Rainy day
        assert!(super::shared::create_shared_byte_data_block(u64::MAX, context).is_err());
    })]);
}

#[test]
fn resize() {
    run_test_actions([TestAction::inspect_context(|context| {
        let data_block = super::create_byte_data_block(100, None, context).unwrap();
        let js_arr = JsArrayBuffer::from_byte_block(data_block, context)
            .unwrap()
            .with_max_byte_length(100);
        let mut arr = js_arr.borrow_mut();

        // Sunny day
        assert_eq!(arr.data_mut().resize(50), Ok(()));

        // Rainy day
        assert!(arr.data_mut().resize(u64::MAX).is_err());
    })]);
}

#[test]
fn resize_with_an_unallocatable_host_limit_preserves_bytes() {
    run_test_actions([TestAction::inspect_context(|context| {
        let bytes = super::create_byte_data_block(1, None, context).unwrap();
        let array = JsArrayBuffer::from_byte_block(bytes, context)
            .unwrap()
            .with_max_byte_length(u64::MAX);
        let mut array = array.borrow_mut();
        let buffer = array.data_mut();

        assert!(buffer.resize(u64::MAX).is_err());
        assert_eq!(buffer.bytes(), Some([0].as_slice()));
    })]);
}

#[test]
fn get_values() {
    run_test_actions([
        TestAction::run(
            r#"
            var buffer = new ArrayBuffer(12);
            var sample = new DataView(buffer, 0);

            sample.setUint8(0, 127);
            sample.setUint8(1, 255);
            sample.setUint8(2, 255);
            sample.setUint8(3, 255);
            sample.setUint8(4, 128);
            sample.setUint8(5, 0);
            sample.setUint8(6, 0);
            sample.setUint8(7, 0);
            sample.setUint8(8, 1);
            sample.setUint8(9, 0);
            sample.setUint8(10, 0);
            sample.setUint8(11, 0);
        "#,
        ),
        TestAction::assert("sample.getUint32(0, false) == 2147483647"),
        TestAction::assert("sample.getUint32(1, false) == 4294967168"),
        TestAction::assert("sample.getUint32(2, false) == 4294934528"),
        TestAction::assert("sample.getUint32(3, false) == 4286578688"),
        TestAction::assert("sample.getUint32(4, false) == 2147483648"),
        TestAction::assert("sample.getUint32(5, false) == 1"),
        TestAction::assert("sample.getUint32(6, false) == 256"),
        TestAction::assert("sample.getUint32(7, false) == 65536"),
        TestAction::assert("sample.getUint32(8, false) == 16777216"),
        TestAction::assert("sample.getUint32(0, true) == 4294967167"),
        TestAction::assert("sample.getUint32(1, true) == 2164260863"),
        TestAction::assert("sample.getUint32(2, true) == 8454143"),
        TestAction::assert("sample.getUint32(3, true) == 33023"),
        TestAction::assert("sample.getUint32(4, true) == 128"),
        TestAction::assert("sample.getUint32(5, true) == 16777216"),
        TestAction::assert("sample.getUint32(6, true) == 65536"),
        TestAction::assert("sample.getUint32(7, true) == 256"),
        TestAction::assert("sample.getUint32(8, true) == 1"),
    ]);
}

#[test]
fn sort() {
    run_test_actions([
        TestAction::run(
            r#"
            // This cmp function is needed as the harness does not support TypedArray comparison.
            function cmp(a, b) {
                return a.length === b.length && a.every((v, i) => v === b[i]);
            }

            var TypedArrayCtor = [
                Int8Array,
                Uint8Array,
                Int16Array,
                Uint16Array,
                Int32Array,
                Uint32Array,
                Float32Array,
                Float64Array,
            ];

            var descending = TypedArrayCtor.map((ctor) => new ctor([4, 3, 2, 1]).sort());
            var mixed = TypedArrayCtor.map((ctor) => new ctor([3, 4, 1, 2]).sort());
            var repeating = TypedArrayCtor.map((ctor) => new ctor([0, 1, 1, 2, 3, 3, 4]).sort());
        "#,
        ),
        // Descending
        TestAction::assert("cmp(descending[0], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[1], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[2], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[3], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[4], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[5], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[6], [1, 2, 3, 4])"),
        TestAction::assert("cmp(descending[7], [1, 2, 3, 4])"),
        // Mixed
        TestAction::assert("cmp(mixed[0], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[1], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[2], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[3], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[4], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[5], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[6], [1, 2, 3, 4])"),
        TestAction::assert("cmp(mixed[7], [1, 2, 3, 4])"),
        // Repeating
        TestAction::assert("cmp(repeating[0], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[1], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[2], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[3], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[4], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[5], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[6], [0, 1, 1, 2, 3, 3, 4])"),
        TestAction::assert("cmp(repeating[7], [0, 1, 1, 2, 3, 3, 4])"),
    ]);
}

#[test]
fn sort_negative_zero() {
    run_test_actions([
        TestAction::run(
            r#"
            // This cmp function is needed as the harness does not support TypedArray comparison.
            function cmp(a, b) {
                return a.length === b.length && a.every((v, i) => v === b[i]);
            }

            var TypedArrayCtor = [Float32Array, Float64Array];
            var negativeZero = TypedArrayCtor.map((ctor) => new ctor([1, 0, -0, 2]).sort());
            var infinities = TypedArrayCtor.map((ctor) => new ctor([3, 4, Infinity, -Infinity, 1, 2]).sort());
        "#,
        ),
        TestAction::assert("cmp(negativeZero[0], [-0, 0, 1, 2])"),
        TestAction::assert("cmp(negativeZero[1], [-0, 0, 1, 2])"),
        TestAction::assert("cmp(infinities[0], [-Infinity, 1, 2, 3, 4, Infinity])"),
        TestAction::assert("cmp(infinities[1], [-Infinity, 1, 2, 3, 4, Infinity])"),
    ]);
}

#[test]
fn shared_byte_access_preserves_unaligned_views_and_logical_growth() {
    run_test_actions([
        TestAction::run(
            r#"
            var backing = new SharedArrayBuffer(9, {maxByteLength: 13});
            var view = new DataView(backing);
            var bytes = new Uint8Array(backing);
            view.setUint32(5, 0x01020304, true);
            var little = view.getUint32(5, true);
            var big = view.getUint32(5, false);
            var beforeGrowth = bytes.join(',');
            backing.grow(13);
            var afterGrowth = bytes.join(',');
            view.setUint16(11, 0xabcd);
            var rejectedPadding = false;
            try { view.getUint16(12); }
            catch (error) { rejectedPadding = error instanceof RangeError; }
            "#,
        ),
        TestAction::assert("little === 0x01020304 && big === 0x04030201"),
        TestAction::assert("beforeGrowth === '0,0,0,0,0,4,3,2,1'"),
        TestAction::assert("afterGrowth === '0,0,0,0,0,4,3,2,1,0,0,0,0'"),
        TestAction::assert("bytes[11] === 0xab && bytes[12] === 0xcd && rejectedPadding"),
    ]);
}

#[test]
fn shared_array_buffer_slice_distinguishes_empty_backing_stores() {
    run_test_actions([
        TestAction::run(
            r#"
            var empty = new SharedArrayBuffer(0);
            var sliced = empty.slice(0);
            empty.constructor = { [Symbol.species]: function () { return empty; } };
            var rejectsAlias = false;
            try { empty.slice(0); } catch (error) { rejectsAlias = error instanceof TypeError; }
            "#,
        ),
        TestAction::assert("sliced !== empty && sliced.byteLength === 0"),
        TestAction::assert("rejectsAlias"),
    ]);
}

/// Tests copying a shared buffer into the distinct backing allocated by `slice`.
#[test]
fn shared_array_buffer_slice() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(16);
            var view = new Uint8Array(sab);
            for (var i = 0; i < 16; i++) view[i] = i + 1;
            var sliced = sab.slice(0);
            var result = new Uint8Array(sliced);
        "#,
        ),
        TestAction::assert("result[0] === 1"),
        TestAction::assert("result[7] === 8"),
        TestAction::assert("result[15] === 16"),
        TestAction::assert("result.length === 16"),
    ]);
}

/// Tests `SharedArrayBuffer.prototype.slice` with an odd offset and length.
#[test]
fn shared_array_buffer_slice_partial() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(20);
            var view = new Uint8Array(sab);
            for (var i = 0; i < 20; i++) view[i] = i * 3;

            var sliced = sab.slice(3, 14);
            var result = new Uint8Array(sliced);
        "#,
        ),
        TestAction::assert("result.length === 11"),
        TestAction::assert("result[0] === 9"),
        TestAction::assert("result[10] === 39"),
    ]);
}

#[test]
fn shared_typed_array_slice_reads_aliased_bytes_in_forward_order() {
    run_test_actions([
        TestAction::run(
            r#"
            var backing = new SharedArrayBuffer(6);
            var source = new Uint8Array(backing);
            source.set([10, 20, 30, 40, 50, 60]);
            source.constructor = {
                [Symbol.species]: function () { return new Uint8Array(backing, 2); }
            };
            var result = source.slice(1, 4);
            "#,
        ),
        TestAction::assert("result.buffer === backing"),
        TestAction::assert("result.join(',') === '20,20,20,60'"),
    ]);
}

/// Tests TypedArray.set from shared to ordinary backing.
#[test]
fn shared_to_regular_typed_array_set() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(16);
            var src = new Uint8Array(sab);
            for (var i = 0; i < 16; i++) src[i] = 100 + i;

            var ab = new ArrayBuffer(16);
            var dest = new Uint8Array(ab);
            dest.set(src);
        "#,
        ),
        TestAction::assert("dest[0] === 100"),
        TestAction::assert("dest[7] === 107"),
        TestAction::assert("dest[15] === 115"),
    ]);
}

/// Tests TypedArray.set from ordinary to shared backing.
#[test]
fn regular_to_shared_typed_array_set() {
    run_test_actions([
        TestAction::run(
            r#"
            var ab = new ArrayBuffer(16);
            var src = new Uint8Array(ab);
            for (var i = 0; i < 16; i++) src[i] = 200 + i;

            var sab = new SharedArrayBuffer(16);
            var dest = new Uint8Array(sab);
            dest.set(src);
        "#,
        ),
        TestAction::assert("dest[0] === 200"),
        TestAction::assert("dest[7] === 207"),
        TestAction::assert("dest[15] === 215"),
    ]);
}

/// Tests forward `copyWithin` on a shared backing.
#[test]
fn shared_typed_array_copy_within() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(16);
            var arr = new Uint8Array(sab);
            for (var i = 0; i < 16; i++) arr[i] = i + 1;

            // Forward copy: copies bytes 4..12 to offset 0
            arr.copyWithin(0, 4, 12);
        "#,
        ),
        TestAction::assert("arr[0] === 5"),
        TestAction::assert("arr[7] === 12"),
        TestAction::assert("arr[8] === 9"),
    ]);
}

/// Tests backward `copyWithin` on a shared backing when the ranges overlap.
#[test]
fn shared_typed_array_copy_within_backward() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(16);
            var arr = new Uint8Array(sab);
            for (var i = 0; i < 16; i++) arr[i] = i + 1;

            // Backward copy: copies bytes 0..8 to offset 4 (overlapping)
            arr.copyWithin(4, 0, 8);
        "#,
        ),
        TestAction::assert("arr[0] === 1"),
        TestAction::assert("arr[3] === 4"),
        TestAction::assert("arr[4] === 1"),
        TestAction::assert("arr[11] === 8"),
        TestAction::assert("arr[12] === 13"),
    ]);
}

/// Tests a zero-length slice of a nonempty shared buffer.
#[test]
fn shared_array_buffer_slice_empty() {
    run_test_actions([
        TestAction::run(
            r#"
            var sab = new SharedArrayBuffer(16);
            var view = new Uint8Array(sab);
            for (var i = 0; i < 16; i++) view[i] = i + 1;
            var sliced = sab.slice(5, 5);
            var result = new Uint8Array(sliced);
        "#,
        ),
        TestAction::assert("result.length === 0"),
    ]);
}
