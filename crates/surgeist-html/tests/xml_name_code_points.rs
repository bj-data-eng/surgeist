//! XML 1.0 Fifth Edition productions [4] and [4a], retained in
//! references/xml-pseudo-attribute-grammar--REC-xml-20081126--f58a4b9e1e5b.md.
#![forbid(unsafe_code)]

use surgeist_html::bounded::{is_xml_name_code_point, is_xml_name_start_code_point};

#[test]
fn xml_name_start_includes_selected_range_edges_and_excludes_the_gaps() {
    for code_point in [
        0x3a, 0x41, 0x5a, 0x5f, 0x61, 0x7a, 0xc0, 0xd6, 0xd8, 0xf6, 0xf8, 0x2ff, 0x370, 0x37d,
        0x37f, 0x1fff, 0x200c, 0x200d, 0x2070, 0x218f, 0x2c00, 0x2fef, 0x3001, 0xd7ff, 0xf900,
        0xfdcf, 0xfdf0, 0xfffd, 0x10000, 0xeffff,
    ] {
        assert!(is_xml_name_start_code_point(code_point), "U+{code_point:X}");
    }
    for code_point in [
        0,
        0x20,
        0x2f,
        0x30,
        0x3e,
        0x40,
        0x5b,
        0x60,
        0x7b,
        0xbf,
        0xd7,
        0xf7,
        0x300,
        0x36f,
        0x37e,
        0x2000,
        0x200b,
        0x200e,
        0x206f,
        0x2190,
        0x2bff,
        0x2ff0,
        0x3000,
        0xd800,
        0xdfff,
        0xf8ff,
        0xfdd0,
        0xfdef,
        0xfffe,
        0xffff,
        0xf0000,
        0x10ffff,
        0x110000,
        u32::MAX,
    ] {
        assert!(
            !is_xml_name_start_code_point(code_point),
            "U+{code_point:X}"
        );
    }
}

#[test]
fn xml_name_continuations_admit_only_the_additional_xml_characters() {
    for code_point in [0x2d, 0x2e, 0x30, 0x39, 0xb7, 0x300, 0x36f, 0x203f, 0x2040] {
        assert!(
            !is_xml_name_start_code_point(code_point),
            "U+{code_point:X}"
        );
        assert!(is_xml_name_code_point(code_point), "U+{code_point:X}");
    }
    for code_point in [0x3a, 0x41, 0x5f, 0x370, 0x10000, 0xeffff] {
        assert!(is_xml_name_code_point(code_point), "U+{code_point:X}");
    }
    for code_point in [
        0x20, 0x2f, 0x3e, 0xb6, 0xb8, 0x37e, 0x203e, 0x2041, 0xd800, 0xf0000,
    ] {
        assert!(!is_xml_name_code_point(code_point), "U+{code_point:X}");
    }
}
