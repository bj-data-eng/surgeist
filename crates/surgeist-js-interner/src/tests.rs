use crate::{COMMON_STRINGS_UTF8, COMMON_STRINGS_UTF16, Interner, Sym};
use surgeist_js_macros::utf16;

#[track_caller]
fn sym_from_usize(index: usize) -> Sym {
    Sym::new(index).expect("Invalid NonZeroUsize")
}

#[test]
fn check_static_strings() {
    let mut interner = Interner::default();

    for (i, &str) in COMMON_STRINGS_UTF8.iter().enumerate() {
        assert_eq!(interner.get_or_intern(str), sym_from_usize(i + 1));
    }
}

#[test]
fn check_new_string() {
    let mut interner = Interner::default();

    assert!(interner.get_or_intern("my test string").get() > COMMON_STRINGS_UTF8.len());
}

#[test]
fn check_resolve() {
    let mut interner = Interner::default();

    let utf_8_strings = ["test string", "arguments", "hello"];
    let utf_8_strings = utf_8_strings.into_iter();
    let utf_16_strings = [utf16!("TEST STRING"), utf16!("ARGUMENTS"), utf16!("HELLO")];
    let utf_16_strings = utf_16_strings.into_iter();

    for (s8, s16) in utf_8_strings.zip(utf_16_strings) {
        let sym = interner.get_or_intern(s8);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(Some(s8), resolved.utf8());
        let new_sym = interner.get_or_intern(s8);
        assert_eq!(sym, new_sym);

        let sym = interner.get_or_intern(s16);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(s16, resolved.utf16());
        let new_sym = interner.get_or_intern(s16);
        assert_eq!(sym, new_sym);
    }
}

#[test]
fn check_static_resolve() {
    let mut interner = Interner::default();

    for (utf8, utf16) in COMMON_STRINGS_UTF8
        .iter()
        .copied()
        .zip(COMMON_STRINGS_UTF16.iter().copied())
        .chain([
            ("my test str", utf16!("my test str")),
            ("hello world", utf16!("hello world")),
            (";", utf16!(";")),
        ])
    {
        let sym = interner.get_or_intern_static(utf8, utf16);
        let resolved = interner.resolve(sym).unwrap();
        assert_eq!(Some(utf8), resolved.utf8());
        assert_eq!(utf16, resolved.utf16());

        let new_sym = interner.get_or_intern(utf8);

        assert_eq!(sym, new_sym);
    }
}

#[test]
#[should_panic(expected = "static UTF-8 and UTF-16 spellings must match")]
fn rejects_mismatched_static_encodings() {
    let mut interner = Interner::new();
    interner.get_or_intern("dynamic static input");
    interner.get_or_intern_static("dynamic static input", utf16!("different input"));
}

#[test]
fn check_unpaired_surrogates() {
    let mut interner = Interner::default();

    let unp = &[
        0xDC15u16, 0xDC19, 'h' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 'o' as u16,
    ];
    let unp2 = &[
        0xDC01u16, 'w' as u16, 'o' as u16, 'r' as u16, 0xDCF4, 'l' as u16, 'd' as u16,
    ];

    let sym = interner.get_or_intern("abc");
    let sym2 = interner.get_or_intern("def");

    let sym3 = interner.get_or_intern(unp);
    let sym4 = interner.get_or_intern(utf16!("ghi"));
    let sym5 = interner.get_or_intern(unp2);

    let sym6 = interner.get_or_intern("jkl");

    assert_eq!(interner.resolve_expect(sym).utf8(), Some("abc"));
    assert_eq!(interner.resolve_expect(sym).utf16(), utf16!("abc"));

    assert_eq!(interner.resolve_expect(sym2).utf8(), Some("def"));
    assert_eq!(interner.resolve_expect(sym2).utf16(), utf16!("def"));

    assert!(interner.resolve_expect(sym3).utf8().is_none());
    assert_eq!(interner.resolve_expect(sym3).utf16(), unp);

    assert_eq!(interner.resolve_expect(sym4).utf8(), Some("ghi"));
    assert_eq!(interner.resolve_expect(sym4).utf16(), utf16!("ghi"));

    assert!(interner.resolve_expect(sym5).utf8().is_none());
    assert_eq!(interner.resolve_expect(sym5).utf16(), unp2);

    assert_eq!(interner.resolve_expect(sym6).utf8(), Some("jkl"));
    assert_eq!(interner.resolve_expect(sym6).utf16(), utf16!("jkl"));
}

#[test]
fn symbol_ids_and_encodings_survive_storage_growth() {
    let mut interner = Interner::with_capacity(1);
    let first_dynamic = interner.len() + 1;
    let lone_high = [0xD800];
    let lone_low = [0xDC00];
    let mixed = "dynamic-λ-😀";
    let latin1 = "dynamic-café";
    let mixed_utf16 = utf16!("dynamic-λ-😀");
    let latin1_utf16 = utf16!("dynamic-café");
    let high = interner.get_or_intern(&lone_high);
    let from_utf16 = interner.get_or_intern(mixed_utf16);
    let low = interner.get_or_intern(&lone_low);
    let from_utf8 = interner.get_or_intern(latin1);
    let empty = interner.get_or_intern("");

    for index in 0..128 {
        interner.get_or_intern(alloc::format!("growth-{index}-longer-name").as_str());
    }

    assert_eq!(high.get(), first_dynamic);
    assert_eq!(from_utf16.get(), first_dynamic + 1);
    assert_eq!(low.get(), first_dynamic + 2);
    assert_eq!(from_utf8.get(), first_dynamic + 3);
    assert_eq!(interner.len(), first_dynamic + 131);
    assert_eq!(interner.get(&lone_high), Some(high));
    assert_eq!(interner.get(&lone_low), Some(low));
    assert_eq!(interner.get(mixed), Some(from_utf16));
    assert_eq!(interner.get(latin1_utf16), Some(from_utf8));
    assert_eq!(interner.get(utf16!("")), Some(empty));
    assert_eq!(interner.resolve_expect(high).utf8(), None);
    assert_eq!(interner.resolve_expect(high).utf16(), lone_high);
    assert_eq!(interner.resolve_expect(low).utf8(), None);
    assert_eq!(interner.resolve_expect(low).utf16(), lone_low);
    assert_eq!(interner.resolve_expect(from_utf16).utf8(), Some(mixed));
    assert_eq!(interner.resolve_expect(from_utf8).utf16(), latin1_utf16);
    assert_eq!(interner.resolve_expect(empty).utf8(), Some(""));
    assert!(!interner.is_latin1(from_utf16));
    assert!(interner.is_latin1(from_utf8));
}

#[test]
fn check_empty_interner() {
    let interner = Interner::default();

    let sym = sym_from_usize(123); // Choose an arbitrary symbol

    assert!(interner.resolve(sym).is_none());
}

#[test]
fn check_capacity() {
    let interner = Interner::with_capacity(100);

    let sym = sym_from_usize(123); // Choose an arbitrary symbol

    assert!(interner.resolve(sym).is_none());
}

#[test]
fn check_is_latin1() {
    static STATIC_STR: &str = "static_latin1";
    static STATIC_UTF16: &[u16] = &[
        's' as u16, 't' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'c' as u16, '_' as u16,
        'l' as u16, 'a' as u16, 't' as u16, 'i' as u16, 'n' as u16, '1' as u16,
    ];

    let mut interner = Interner::default();

    // Common/static strings (e.g. keywords) are always Latin1.
    let common_sym = interner.get_or_intern("break");
    assert!(interner.is_latin1(common_sym));

    // Dynamic ASCII string.
    let ascii_sym = interner.get_or_intern("hello_world");
    assert!(interner.is_latin1(ascii_sym));

    // Dynamic non-ASCII but Latin1-encodable (U+0080..=U+00FF).
    let latin1_sym = interner.get_or_intern(&[0x00E9u16, 0x00FC, 0x00F1][..]);
    assert!(interner.is_latin1(latin1_sym));

    // Dynamic non-Latin1 (code unit > 0xFF).
    let non_latin1_sym = interner.get_or_intern(&[0x4E2Du16, 0x6587][..]);
    assert!(!interner.is_latin1(non_latin1_sym));

    // Boundary: U+00FF (last Latin1) and U+0100 (first non-Latin1).
    let boundary_sym = interner.get_or_intern(&[0x00FFu16][..]);
    assert!(interner.is_latin1(boundary_sym));
    let boundary_non_sym = interner.get_or_intern(&[0x0100u16][..]);
    assert!(!interner.is_latin1(boundary_non_sym));

    // get_or_intern_static also caches correctly.
    let static_sym = interner.get_or_intern_static(STATIC_STR, STATIC_UTF16);
    assert!(interner.is_latin1(static_sym));
}
