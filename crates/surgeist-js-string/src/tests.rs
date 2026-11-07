#![allow(clippy::redundant_clone)]

use std::hash::{BuildHasher, BuildHasherDefault, Hash};

use crate::{
    CodePoint, CommonJsStringBuilder, JsStr, JsString, JsStringKind, Latin1JsStringBuilder,
    StaticJsStrings, StaticString, Utf16JsStringBuilder,
};

use rustc_hash::FxHasher;

fn hash_value<T: Hash>(value: &T) -> u64 {
    BuildHasherDefault::<FxHasher>::default().hash_one(value)
}

const fn ascii_to_utf16<const LEN: usize>(ascii: &[u8; LEN]) -> [u16; LEN] {
    let mut array = [0; LEN];
    let mut i = 0;
    while i < LEN {
        array[i] = ascii[i] as u16;
        i += 1;
    }
    array
}

#[test]
fn empty() {
    let s = StaticJsStrings::EMPTY_STRING;
    assert_eq!(&s, &[]);
}

#[test]
fn refcount() {
    let x = JsString::from("Hello world");
    assert_eq!(x.refcount(), Some(1));

    {
        let y = x.clone();
        assert_eq!(x.refcount(), Some(2));
        assert_eq!(y.refcount(), Some(2));

        {
            let z = y.clone();
            assert_eq!(x.refcount(), Some(3));
            assert_eq!(y.refcount(), Some(3));
            assert_eq!(z.refcount(), Some(3));
        }

        assert_eq!(x.refcount(), Some(2));
        assert_eq!(y.refcount(), Some(2));
    }

    assert_eq!(x.refcount(), Some(1));
}

#[test]
fn static_refcount() {
    let x = StaticJsStrings::EMPTY_STRING;
    assert_eq!(x.refcount(), None);

    {
        let y = x.clone();
        assert_eq!(x.refcount(), None);
        assert_eq!(y.refcount(), None);
    };

    assert_eq!(x.refcount(), None);
}

fn same_storage(left: &JsString, right: &JsString) -> bool {
    use crate::storage::{Root, Storage};
    match (&left.storage, &right.storage) {
        (Storage::Root(Root::Static(a)), Storage::Root(Root::Static(b))) => std::ptr::eq(*a, *b),
        (Storage::Root(Root::Sequence(a)), Storage::Root(Root::Sequence(b))) => {
            std::rc::Rc::ptr_eq(a, b)
        }
        _ => false,
    }
}

#[test]
fn ptr_eq() {
    let x = JsString::from("Hello");
    let y = x.clone();

    assert!(!x.is_static());

    assert!(same_storage(&x, &y));

    let z = JsString::from("Hello");
    assert!(!same_storage(&x, &z));
    assert!(!same_storage(&y, &z));
}

#[test]
fn static_ptr_eq() {
    let x = StaticJsStrings::EMPTY_STRING;
    let y = x.clone();

    assert!(x.is_static());

    assert!(same_storage(&x, &y));

    let z = StaticJsStrings::EMPTY_STRING;
    assert!(same_storage(&x, &z));
    assert!(same_storage(&y, &z));
}

#[test]
fn as_str() {
    const HELLO: &[u16] = &ascii_to_utf16(b"Hello");
    let x = JsString::from(HELLO);

    assert_eq!(&x, HELLO);
}

#[test]
fn hash() {
    const HELLOWORLD: JsStr<'_> = JsStr::latin1("Hello World!".as_bytes());
    let x = JsString::from(HELLOWORLD);

    assert_eq!(x.as_str(), HELLOWORLD);

    assert!(HELLOWORLD.is_latin1());
    assert!(x.as_str().is_latin1());

    let s_hash = hash_value(&HELLOWORLD);
    let x_hash = hash_value(&x);

    assert_eq!(s_hash, x_hash);
}

#[test]
fn concat() {
    const Y: &[u16] = &ascii_to_utf16(b", ");
    const W: &[u16] = &ascii_to_utf16(b"!");

    let x = JsString::from("hello");
    let z = JsString::from("world");

    let xy = JsString::concat(x.as_str(), JsString::from(Y).as_str());
    assert_eq!(&xy, &ascii_to_utf16(b"hello, "));
    assert_eq!(xy.refcount(), Some(1));

    let xyz = JsString::concat(xy.as_str(), z.as_str());
    assert_eq!(&xyz, &ascii_to_utf16(b"hello, world"));
    assert_eq!(xyz.refcount(), Some(1));

    let xyzw = JsString::concat(xyz.as_str(), JsString::from(W).as_str());
    assert_eq!(&xyzw, &ascii_to_utf16(b"hello, world!"));
    assert_eq!(xyzw.refcount(), Some(1));
}

#[test]
fn trim_start_non_ascii_to_ascii() {
    let s = "\u{2029}abc";
    let x = JsString::from(s);

    let y = x.trim_start();

    assert_eq!(&y, s.trim_start());
}

#[test]
fn conversion_to_known_static_js_string() {
    const JS_STR_U8: &JsStr<'_> = &JsStr::latin1("length".as_bytes());
    const JS_STR_U16: &JsStr<'_> = &JsStr::utf16(&ascii_to_utf16(b"length"));

    assert!(JS_STR_U8.is_latin1());
    assert!(!JS_STR_U16.is_latin1());

    assert_eq!(JS_STR_U8, JS_STR_U8);
    assert_eq!(JS_STR_U16, JS_STR_U16);

    assert_eq!(JS_STR_U8, JS_STR_U16);
    assert_eq!(JS_STR_U16, JS_STR_U8);

    assert_eq!(hash_value(JS_STR_U8), hash_value(JS_STR_U16));

    let string = StaticJsStrings::get_string(JS_STR_U8);

    assert!(string.is_some());
    assert!(string.unwrap().as_str().is_latin1());

    let string = StaticJsStrings::get_string(JS_STR_U16);

    assert!(string.is_some());
    assert!(string.unwrap().as_str().is_latin1());
}

#[test]
fn to_std_string_escaped() {
    assert_eq!(
        JsString::from("Hello, \u{1D49E} world!").to_std_string_escaped(),
        "Hello, \u{1D49E} world!"
    );

    assert_eq!(
        JsString::from("Hello, world!").to_std_string_escaped(),
        "Hello, world!"
    );

    // 15 should not be escaped.
    let unpaired_surrogates: [u16; 3] = [0xDC58, 0xD83C, 0x0015];
    assert_eq!(
        JsString::from(&unpaired_surrogates).to_std_string_escaped(),
        "\\uDC58\\uD83C\u{15}"
    );
}

#[test]
fn from_static_js_string() {
    static STATIC_HELLO_WORLD: StaticString =
        StaticString::new(JsStr::latin1("hello world".as_bytes()));
    static STATIC_EMOJIS: StaticString = StaticString::new(JsStr::utf16(&[
        0xD83C, 0xDFB9, 0xD83C, 0xDFB6, 0xD83C, 0xDFB5,
    ])); // 🎹🎶🎵

    let latin1 = JsString::from_static(&STATIC_HELLO_WORLD);
    let utf16 = JsString::from_static(&STATIC_EMOJIS);

    // content compare
    assert_eq!(latin1, "hello world");
    assert_eq!(utf16, "🎹🎶🎵");

    // refcount check
    let clone = latin1.clone();

    assert_eq!(clone, latin1);

    let clone = utf16.clone();

    assert_eq!(clone, utf16);

    assert!(latin1.refcount().is_none());
    assert!(utf16.refcount().is_none());

    // `is_latin1` check
    assert!(latin1.as_str().is_latin1());
    assert!(!utf16.as_str().is_latin1());
}

#[test]
fn compare_static_and_dynamic_js_string() {
    static STATIC_HELLO_WORLD: StaticString =
        StaticString::new(JsStr::latin1("hello world".as_bytes()));
    static STATIC_EMOJIS: StaticString = StaticString::new(JsStr::utf16(&[
        0xD83C, 0xDFB9, 0xD83C, 0xDFB6, 0xD83C, 0xDFB5,
    ])); // 🎹🎶🎵

    let static_latin1 = JsString::from_static(&STATIC_HELLO_WORLD);
    let static_utf16 = JsString::from_static(&STATIC_EMOJIS);

    let dynamic_latin1 = JsString::from(JsStr::latin1("hello world".as_bytes()));
    let dynamic_utf16 = JsString::from(&[0xD83C, 0xDFB9, 0xD83C, 0xDFB6, 0xD83C, 0xDFB5]);

    // content compare
    assert_eq!(static_latin1, dynamic_latin1);
    assert_eq!(static_utf16, dynamic_utf16);

    // length check
    assert_eq!(static_latin1.len(), dynamic_latin1.len());
    assert_eq!(static_utf16.len(), dynamic_utf16.len());

    // `is_static` check
    assert!(static_latin1.is_static());
    assert!(static_utf16.is_static());
    assert!(!dynamic_latin1.is_static());
    assert!(!dynamic_utf16.is_static());
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn js_string_builder() {
    let s = "2024年5月21日";
    let utf16 = s.encode_utf16().collect::<Vec<_>>();
    let s_utf16 = utf16.as_slice();
    let ascii = "Lorem ipsum dolor sit amet";
    let s_ascii = ascii.as_bytes();
    let latin1_as_utf8_literal = "Déjà vu";
    let s_latin1_literal: &[u8] = &[
        b'D', 0xE9, /* é */
        b'j', 0xE0, /* à */
        b' ', b'v', b'u',
    ];

    // latin1 builder -- test

    // push ascii
    let mut builder = Latin1JsStringBuilder::new();
    for &code in s_ascii {
        builder.push(code);
    }
    let s_builder = builder.build().unwrap_or_default();
    assert_eq!(s_builder, ascii);

    // push latin1
    let mut builder = Latin1JsStringBuilder::new();
    for &code in s_latin1_literal {
        builder.push(code);
    }
    let s_builder = builder.build_as_latin1();
    assert_eq!(
        s_builder.to_std_string().unwrap_or_default(),
        latin1_as_utf8_literal
    );

    // from_iter ascii
    let s_builder = s_ascii
        .iter()
        .copied()
        .collect::<Latin1JsStringBuilder>()
        .build()
        .unwrap_or_default();
    assert_eq!(s_builder.to_std_string().unwrap_or_default(), ascii);

    // from_iter latin1
    let s_builder = s_latin1_literal
        .iter()
        .copied()
        .collect::<Latin1JsStringBuilder>()
        .build_as_latin1();
    assert_eq!(
        s_builder.to_std_string().unwrap_or_default(),
        latin1_as_utf8_literal
    );

    // extend_from_slice ascii
    let mut builder = Latin1JsStringBuilder::new();
    builder.extend_from_slice(s_ascii);
    let s_builder = builder.build().unwrap_or_default();
    assert_eq!(s_builder.to_std_string().unwrap_or_default(), ascii);

    // extend_from_slice latin1
    let mut builder = Latin1JsStringBuilder::new();
    builder.extend_from_slice(s_latin1_literal);
    let s_builder = builder.build_as_latin1();
    assert_eq!(
        s_builder.to_std_string().unwrap_or_default(),
        latin1_as_utf8_literal
    );

    // build from utf16 encoded string
    let s_builder = s
        .as_bytes()
        .iter()
        .copied()
        .collect::<Latin1JsStringBuilder>()
        .build();
    assert_eq!(None, s_builder);

    let s_builder = s_utf16
        .iter()
        .copied()
        .map(|v| v as u8)
        .collect::<Latin1JsStringBuilder>()
        .build();
    assert_eq!(None, s_builder);

    // utf16 builder -- test

    // push
    let mut builder = Utf16JsStringBuilder::new();
    for &code in s_utf16 {
        builder.push(code);
    }
    let s_builder = builder.build();
    assert_eq!(s_builder.to_std_string().unwrap_or_default(), s);

    // from_iter
    let s_builder = s_utf16
        .iter()
        .copied()
        .collect::<Utf16JsStringBuilder>()
        .build();
    assert_eq!(s_builder.to_std_string().unwrap_or_default(), s);

    // extend_from_slice
    let mut builder = Utf16JsStringBuilder::new();
    builder.extend_from_slice(s_utf16);
    let s_builder = builder.build();
    assert_eq!(s_builder.to_std_string().unwrap_or_default(), s);
}

#[test]
fn clone_builder() {
    // latin1 builder -- test
    let origin = Latin1JsStringBuilder::from(&b"0123456789"[..]);
    let empty_origin = Latin1JsStringBuilder::new();

    // clone == origin
    let cloned = origin.clone();
    assert_eq!(origin, cloned);

    // clone_from == origin
    let mut cloned_from = Latin1JsStringBuilder::new();
    cloned_from.clone_from(&origin);
    assert_eq!(origin, cloned_from);

    // clone == origin(empty)
    let cloned = empty_origin.clone();
    assert_eq!(empty_origin, cloned);

    // clone_from == origin(empty)

    cloned_from.clone_from(&empty_origin);
    assert!(cloned_from.capacity() > 0); // Should not be reallocated so the capacity is preserved.
    assert_eq!(empty_origin, cloned_from);

    // clone_from(empty) == origin(empty)
    let mut cloned_from = Latin1JsStringBuilder::new();
    cloned_from.clone_from(&empty_origin);
    assert_eq!(cloned_from.capacity(), 0);
    assert_eq!(empty_origin, cloned_from);

    // utf16 builder -- test
    let s = "2024年5月21日";

    let origin = Utf16JsStringBuilder::from(s.encode_utf16().collect::<Vec<_>>().as_slice());
    let empty_origin = Utf16JsStringBuilder::new();
    // clone == origin
    let cloned = origin.clone();
    assert_eq!(origin, cloned);

    // clone_from == origin(empty)
    let mut cloned_from = Utf16JsStringBuilder::new();
    cloned_from.clone_from(&origin);

    assert_eq!(origin, cloned_from);
    // clone == origin(empty)
    let cloned = empty_origin.clone();
    assert_eq!(empty_origin, cloned);

    // clone_from == origin(empty)

    cloned_from.clone_from(&empty_origin);
    assert!(cloned_from.capacity() > 0); // should not be reallocated so the capacity is preserved.
    assert_eq!(empty_origin, cloned_from);

    // clone_from(empty) == origin(empty)
    let mut cloned_from = Utf16JsStringBuilder::new();
    cloned_from.clone_from(&empty_origin);
    assert_eq!(cloned_from.capacity(), 0);
    assert_eq!(empty_origin, cloned_from);
}

#[test]
fn common_js_string_builder() {
    let utf16 = "2024年5月21日".encode_utf16().collect::<Vec<_>>();
    let s_utf16 = utf16.as_slice();
    let s = "Lorem ipsum dolor sit amet";
    let js_str_utf16 = JsStr::utf16(s_utf16);
    let js_str_ascii = JsStr::latin1(s.as_bytes());
    let latin1_bytes = [
        b'D', 0xE9, /* é */
        b'j', 0xE0, /* à */
        b' ', b'v', b'u',
    ];
    let ch = '🎹';
    let mut builder = CommonJsStringBuilder::with_capacity(10);
    builder += ch;
    builder += s;
    builder += js_str_utf16;
    builder += js_str_ascii;
    builder += ch;
    assert_eq!(builder.len(), 5);
    let js_string = builder.build_from_utf16();
    assert_eq!(
        js_string,
        "🎹Lorem ipsum dolor sit amet2024年5月21日Lorem ipsum dolor sit amet🎹"
    );
    let mut builder = CommonJsStringBuilder::new();
    for b in latin1_bytes {
        builder += b;
    }
    builder += s_utf16;
    builder += ch;
    let js_string = builder.build();
    assert_eq!(
        js_string.to_std_string().unwrap_or_default(),
        "Déjà vu2024年5月21日🎹"
    );
}

#[test]
fn code_points_optimization() {
    // Test Latin1 optimization with extended Latin1 characters
    let latin1_str = JsStr::latin1(b"Caf\xe9 na\xefve"); // "Café naïve" in Latin1 encoding
    let latin1_points: Vec<CodePoint> = latin1_str.code_points().collect();
    let expected_latin1: Vec<CodePoint> = "Café naïve".chars().map(CodePoint::Unicode).collect();
    assert_eq!(latin1_points, expected_latin1);

    // Test UTF-16 behavior unchanged (including non-ASCII)
    let utf16_str = JsStr::utf16(&[
        0x0043, 0x0061, 0x0066, 0x00E9, // "Café"
        0x0020, // space
        0x006E, 0x0061, 0x00EF, 0x0076, 0x0065, // "naïve"
    ]);
    let utf16_points: Vec<CodePoint> = utf16_str.code_points().collect();
    assert_eq!(latin1_points, utf16_points); // Same result for same content
}

#[test]
fn slice() {
    let sliced = {
        let base_str = JsString::from("Hello World");
        assert_eq!(base_str.kind(), JsStringKind::Latin1Sequence);

        base_str.slice(1, 5)
    };
    assert_eq!(sliced, JsString::from("ello"));
    assert_eq!(sliced.kind(), JsStringKind::Slice);

    let sliced2 = sliced.slice(1, 3);
    drop(sliced);
    assert_eq!(sliced2, JsString::from("ll"));
    assert_eq!(sliced2.kind(), JsStringKind::Slice);

    let sliced3 = sliced2.slice(0, 2);
    drop(sliced2);
    assert_eq!(sliced3, JsString::from("ll"));
    assert_eq!(sliced3.kind(), JsStringKind::Slice);

    let sliced4 = sliced3.slice(0, 2);
    drop(sliced3);
    assert_eq!(sliced4, JsString::from("ll"));
    assert_eq!(sliced4.kind(), JsStringKind::Slice);

    let sliced4 = sliced4.slice(0, 2);
    assert_eq!(sliced4, JsString::from("ll"));
    assert_eq!(sliced4.kind(), JsStringKind::Slice);

    let sliced5 = sliced4.slice(1, 1);
    assert_eq!(sliced5, JsString::from(""));
    assert_eq!(sliced5.kind(), JsStringKind::Static);

    assert_eq!(sliced5.slice(4, 4), StaticJsStrings::EMPTY_STRING);
}

#[test]
fn inclusive_string_range_overflow_is_out_of_bounds() {
    let string = JsString::from("slice input");
    assert!(string.get(..=usize::MAX).is_none());
    assert!(string.get(1..=usize::MAX).is_none());
}

#[test]
fn nested_utf16_slices_preserve_backing_units_and_parent_counts() {
    let base = JsString::from(&[0xD83D, 0xDE00, 0x0061, 0xD800]);
    let parent = base.get_expect(1..3);
    let child = parent.get_expect(0..1);

    assert_eq!(base.refcount(), Some(2));
    assert_eq!(parent.refcount(), Some(2));
    assert_eq!(child.refcount(), Some(1));
    let expected = base.as_str().get_expect(1..2);
    let (crate::JsStrVariant::Utf16(child_units), crate::JsStrVariant::Utf16(base_units)) =
        (child.variant(), expected.variant())
    else {
        panic!("nested slices must keep UTF-16 storage");
    };
    assert_eq!(child_units, [0xDE00]);
    assert!(std::ptr::eq(child_units, base_units));
    assert_eq!(
        format!("{:?}", child.debug_info())
            .matches("original")
            .count(),
        2
    );

    let empty = child.get_expect(0..0);
    assert_eq!(child.refcount(), Some(2));
    assert_eq!(empty.kind(), JsStringKind::Slice);
    assert!(!empty.as_str().is_latin1());
    assert_eq!(child.slice(0, 0).kind(), JsStringKind::Static);
    drop((base, parent, child));
    assert!(empty.is_empty());
}

#[test]
fn dropping_deep_slice_chain_keeps_shared_ancestor() {
    let base = JsString::from("dynamic slice chain");
    let ancestor = base.get_expect(..);
    let mut leaf = ancestor.clone();
    for _ in 0..50_000 {
        leaf = leaf.get_expect(..);
    }
    assert_eq!(leaf.as_str(), ancestor.as_str());
    assert_eq!(ancestor.refcount(), Some(2));
    drop(leaf);
    assert_eq!(ancestor.refcount(), Some(1));
    assert_eq!(ancestor.as_str(), base.as_str());
    drop(ancestor);
    assert_eq!(base.refcount(), Some(1));
}

#[test]
fn split() {
    let base_str = JsString::from("Hello World");
    assert_eq!(base_str.kind(), JsStringKind::Latin1Sequence);

    let str1 = base_str.slice(0, 5);
    let str2 = base_str.slice(6, base_str.len());

    assert_eq!(str1, JsString::from("Hello"));
    assert_eq!(str2, JsString::from("World"));

    let str3 = str1.clone();
    drop(str1);
    assert_eq!(str3, JsString::from("Hello"));
    drop(base_str);
    assert_eq!(str3, JsString::from("Hello"));
}

#[test]
fn trim() {
    // Very basic test for trimming. The extensive testing is done by `surgeist_js`.
    let base_str = JsString::from(" \u{000B} Hello World \t ");
    assert_eq!(base_str.trim(), JsString::from("Hello World"));
}

#[test]
fn starts_with_and_ends_with_basic() {
    let basic = JsString::from("abcdef");
    let start_needle = JsStr::latin1("abc".as_bytes());
    assert!(basic.starts_with(start_needle));
    assert!(!basic.ends_with(start_needle));

    let end_needle = JsStr::latin1("def".as_bytes());
    assert!(!basic.starts_with(end_needle));
    assert!(basic.ends_with(end_needle));
}

#[test]
fn admitted_copy_concat_and_export_preserve_exact_code_units() {
    use crate::StringWork;
    let mut accept = |_| Ok::<_, ()>(());
    let latin = JsStr::latin1(&[0, 0xE9, 0xFF]);
    let utf16 = JsStr::utf16(&[0xD800, 0x0061, 0xDC00]);
    let copied = JsString::try_from_str(utf16, &mut accept).unwrap();
    assert_eq!(copied.to_vec(), [0xD800, 0x0061, 0xDC00]);
    let mut requests = Vec::new();
    let joined = JsString::try_concat(latin, utf16, &mut |work| {
        requests.push(work);
        Ok::<_, ()>(())
    })
    .unwrap();
    assert_eq!(joined.to_vec(), [0, 0xE9, 0xFF, 0xD800, 0x61, 0xDC00]);
    assert_eq!(
        joined.as_str().try_to_vec(&mut accept).unwrap(),
        [0, 0xE9, 0xFF, 0xD800, 0x61, 0xDC00]
    );
    assert!(requests.contains(&StringWork::Allocate { bytes: 12 }));
    assert_eq!(
        JsString::try_concat(latin, latin, &mut accept)
            .unwrap()
            .as_str()
            .as_latin1(),
        Some(&[0, 0xE9, 0xFF, 0, 0xE9, 0xFF][..])
    );
}

#[test]
fn admitted_utf16_comparison_uses_code_units_across_encodings() {
    use std::cmp::Ordering;
    let mut accept = |_| Ok::<_, ()>(());
    assert!(
        JsStr::latin1(&[0xE9])
            .try_eq(JsStr::utf16(&[0xE9]), &mut accept)
            .unwrap()
    );
    // UTF-16 lexical order compares the high surrogate before the BMP unit.
    assert_eq!(
        JsStr::utf16(&[0xD800, 0xDC00])
            .try_cmp(JsStr::utf16(&[0xE000]), &mut accept)
            .unwrap(),
        Ordering::Less
    );
    assert_eq!(
        JsStr::utf16(&[0xDC00])
            .try_cmp(JsStr::utf16(&[0xD800]), &mut accept)
            .unwrap(),
        Ordering::Greater
    );
    let mut left = vec![b'a'; 600];
    let mut right = vec![u16::from(b'a'); 600];
    left[599] = b'b';
    assert_eq!(
        JsStr::latin1(&left)
            .try_cmp(JsStr::utf16(&right), &mut accept)
            .unwrap(),
        Ordering::Greater
    );
    right[599] = u16::from(b'b');
    assert!(
        JsStr::latin1(&left)
            .try_eq(JsStr::utf16(&right), &mut accept)
            .unwrap()
    );
    assert!(
        !JsStr::latin1(&left[..599])
            .try_eq(JsStr::utf16(&right), &mut accept)
            .unwrap()
    );
}

#[test]
fn admitted_unicode_decode_preserves_scalars_and_rejects_unpaired_surrogates() {
    let mut accept = |_| Ok::<_, ()>(());
    assert_eq!(
        JsStr::latin1(&[0xE9, 0xFF])
            .try_to_std_string(&mut accept)
            .unwrap()
            .as_deref(),
        Some("éÿ")
    );
    assert_eq!(
        JsStr::utf16(&[0xD83D, 0xDE00])
            .try_to_std_string(&mut accept)
            .unwrap()
            .as_deref(),
        Some("😀")
    );
    for units in [&[0xD800][..], &[0xDC00][..], &[0xD800, 0x61][..]] {
        assert!(
            JsStr::utf16(units)
                .try_to_std_string(&mut accept)
                .unwrap()
                .is_none()
        );
        assert!(
            JsStr::utf16(units)
                .try_to_number(&mut accept)
                .unwrap()
                .is_nan()
        );
    }
    assert_eq!(
        JsString::try_from_utf8("é😀", &mut accept)
            .unwrap()
            .to_vec(),
        [0xE9, 0xD83D, 0xDE00]
    );
}

#[test]
fn admitted_string_number_conversion_obeys_numeric_value_boundaries() {
    let mut accept = |_| Ok::<_, ()>(());
    for (text, expected) in [
        (" \t", 0.0),
        ("\u{FEFF}-0\u{2029}", -0.0),
        ("0b101", 5.0),
        ("0o17", 15.0),
        ("0xFFFFFFFF", 4294967295.0),
        ("0x100000000", 4294967296.0),
        (
            "1.00000000000000011102230246251565404236316680908203125",
            1.0,
        ),
        ("5e-324", f64::from_bits(1)),
        ("1.7976931348623157e308", f64::MAX),
        ("Infinity", f64::INFINITY),
        ("-Infinity", f64::NEG_INFINITY),
    ] {
        let string = JsString::from(text);
        let result = string.as_str().try_to_number(&mut accept).unwrap();
        assert_eq!(result.to_bits(), expected.to_bits(), "{text}");
        assert_eq!(
            string.to_number().to_bits(),
            expected.to_bits(),
            "ordinary {text}"
        );
    }
    for text in ["0x", "0b2", "12z", "infinity", "\u{0085}1", "1\u{D7FF}"] {
        assert!(
            JsString::from(text)
                .as_str()
                .try_to_number(&mut accept)
                .unwrap()
                .is_nan(),
            "{text}"
        );
    }
}

#[test]
fn admitted_scalar_formatting_preserves_ecmascript_number_spelling() {
    let mut accept = |_| Ok::<_, ()>(());
    for (number, spelling) in [
        (-0.0, "0"),
        (1e21, "1e+21"),
        (f64::INFINITY, "Infinity"),
        (f64::NAN, "NaN"),
    ] {
        assert_eq!(
            JsString::try_from_number(number, &mut accept).unwrap(),
            spelling
        );
    }
    assert_eq!(
        JsString::try_from_integer(i32::MIN, &mut accept).unwrap(),
        "-2147483648"
    );
}

#[test]
fn admitted_string_refusal_prevents_allocation_and_block_work() {
    use crate::{StringError, StringWork};
    let source = [0xD800; 700];
    let mut copied_units = 0;
    let result = JsString::try_from_str(JsStr::utf16(&source), &mut |work| {
        if let StringWork::Copy { units } = work {
            if copied_units == 256 {
                return Err("second block");
            }
            copied_units += units;
        }
        Ok(())
    });
    assert!(matches!(result, Err(StringError::Refused("second block"))));
    assert_eq!(copied_units, 256);
    assert!(matches!(
        JsString::try_concat(JsStr::latin1(b"a"), JsStr::latin1(b"b"), &mut |work| {
            if matches!(work, StringWork::Allocate { .. }) {
                Err("allocation")
            } else {
                Ok(())
            }
        }),
        Err(StringError::Refused("allocation"))
    ));
    let mut compared = 0;
    let result = JsStr::utf16(&source).try_cmp(JsStr::utf16(&source), &mut |work| {
        if let StringWork::Scan { units } = work {
            if compared == 256 {
                return Err("comparison");
            }
            compared += units;
        }
        Ok(())
    });
    assert!(matches!(result, Err(StringError::Refused("comparison"))));
    assert_eq!(compared, 256);
}

#[test]
fn admitted_decimal_provider_reservation_and_publication_can_be_refused() {
    use crate::{StringError, StringWork};
    let string = JsStr::latin1(b"123.5");
    let mut reserved = false;
    assert!(matches!(
        string.try_to_number(&mut |work| {
            if let StringWork::FastFloat { bytes } = work {
                assert_eq!(bytes, 5);
                reserved = true;
                return Err("provider entry");
            }
            Ok(())
        }),
        Err(StringError::Refused("provider entry"))
    ));
    assert!(reserved);
    let mut entered = false;
    assert!(matches!(
        string.try_to_number(&mut |work| {
            match work {
                StringWork::FastFloat { bytes: 5 } => entered = true,
                StringWork::Checkpoint if entered => return Err("publication"),
                _ => {}
            }
            Ok(())
        }),
        Err(StringError::Refused("publication"))
    ));
}
