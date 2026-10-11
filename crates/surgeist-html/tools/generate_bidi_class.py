#!/usr/bin/env python3
"""Generate the private HTML first-strong property provider, entirely offline.

--write regenerates ordinary Rust; --check verifies reproducibility. Both check
every code point against a separately evaluated range/default lookup.
--verify-unicode-data PATH additionally compares every assigned character with
the pinned independent UnicodeData.txt witness. Unassigned expectations select
the most specific @missing range, then apply explicit property records.

Unicode data is licensed under Unicode License V3; see the repository's Unicode
license/notice material. This tool copies no UAX #9 report prose or tables.
"""

import argparse
from bisect import bisect_right
import hashlib
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[3]
INPUT = Path(__file__).resolve().parent / "data/DerivedBidiClass-18.0.0.txt"
OUTPUT = ROOT / "crates/surgeist-html/src/bidi.rs"
VERSION = "18.0.0"
SOURCE_HASH = "d9e23222522551348ea1ccfbb4f62efbf98982afb95840f8959c08ed992c5607"
UNICODE_DATA_HASH = "0736451de439ae7baf1425136617da495e09ee5afbe6e394374db7009ea08950"
DOMAIN = 0x110000
NAMES = {
    "L": "Left_To_Right", "R": "Right_To_Left", "AL": "Arabic_Letter",
    "EN": "European_Number", "ES": "European_Separator",
    "ET": "European_Terminator", "AN": "Arabic_Number",
    "CS": "Common_Separator", "B": "Paragraph_Separator",
    "S": "Segment_Separator", "WS": "White_Space", "ON": "Other_Neutral",
    "BN": "Boundary_Neutral", "NSM": "Nonspacing_Mark",
    "LRO": "Left_To_Right_Override", "RLO": "Right_To_Left_Override",
    "LRE": "Left_To_Right_Embedding", "RLE": "Right_To_Left_Embedding",
    "PDF": "Pop_Directional_Format", "LRI": "Left_To_Right_Isolate",
    "RLI": "Right_To_Left_Isolate", "FSI": "First_Strong_Isolate",
    "PDI": "Pop_Directional_Isolate",
}
ALIASES = {name: index for index, pair in enumerate(NAMES.items()) for name in pair}
STRONG = {ALIASES["L"]: "L", ALIASES["R"]: "R", ALIASES["AL"]: "Al"}


def pinned_text(path, expected):
    raw = path.read_bytes()
    actual = hashlib.sha256(raw).hexdigest()
    if actual != expected:
        raise ValueError(f"{path}: expected SHA-256 {expected}, got {actual}")
    return raw.decode("utf-8")


def bounds(raw):
    if not re.fullmatch(r"[0-9A-F]{4,6}(?:\.\.[0-9A-F]{4,6})?", raw):
        raise ValueError(f"invalid code-point range {raw!r}")
    ends = raw.split("..")
    first, last = int(ends[0], 16), int(ends[-1], 16)
    if not 0 <= first <= last < DOMAIN:
        raise ValueError(f"range outside code-point domain: {raw}")
    return first, last


def value(raw):
    if raw not in ALIASES:
        raise ValueError(f"unknown Bidi_Class alias {raw!r}")
    return ALIASES[raw]


def disjoint(records):
    previous = -1
    for first, last, _ in sorted(records):
        if first <= previous:
            raise ValueError(f"conflicting/duplicate range beginning U+{first:04X}")
        previous = last


def read_ranges(text):
    if not text.startswith(f"# DerivedBidiClass-{VERSION}.txt\n") or not text.rstrip().endswith("# EOF"):
        raise ValueError("missing pinned version header or complete-file terminator")
    defaults, explicit = [], []
    for number, line in enumerate(text.splitlines(), 1):
        if "@missing:" in line:
            raw = line.partition("@missing:")[2].strip()
            fields = [field.strip() for field in raw.split(";")]
            if len(fields) != 2:
                raise ValueError(f"invalid @missing record at line {number}")
            defaults.append((*bounds(fields[0]), value(fields[1])))
        else:
            raw = line.partition("#")[0].strip()
            if not raw:
                continue
            fields = [field.strip() for field in raw.split(";")]
            if len(fields) != 2:
                raise ValueError(f"invalid explicit record at line {number}")
            explicit.append((*bounds(fields[0]), value(fields[1])))
    if not defaults or defaults[0] != (0, DOMAIN - 1, ALIASES["L"]):
        raise ValueError("missing complete-domain Left_To_Right default")
    disjoint(defaults[1:])
    disjoint(explicit)
    if set(kind for _, _, kind in explicit) != set(ALIASES.values()):
        raise ValueError("incomplete Bidi_Class vocabulary")
    return defaults, sorted(explicit)


def expand(defaults, explicit):
    result = bytearray(DOMAIN)
    for first, last, kind in defaults + explicit:
        result[first:last + 1] = bytes([kind]) * (last - first + 1)
    return result


def assigned_witness(path):
    assigned = bytearray([255]) * DOMAIN
    pending = None
    for line in pinned_text(path, UNICODE_DATA_HASH).splitlines():
        fields = line.split(";")
        if len(fields) != 15:
            raise ValueError("invalid UnicodeData record")
        point, name, kind = int(fields[0], 16), fields[1], value(fields[4])
        if not 0 <= point < DOMAIN:
            raise ValueError("UnicodeData point outside domain")
        if name.endswith(", First>"):
            if pending is not None:
                raise ValueError("nested UnicodeData range")
            pending = (point, name[:-8], kind)
            continue
        if name.endswith(", Last>"):
            if pending is None or (name[:-7], kind) != pending[1:]:
                raise ValueError("unmatched UnicodeData range")
            first = pending[0]
            pending = None
        else:
            if pending is not None:
                raise ValueError("unterminated UnicodeData range")
            first = point
        if first > point or any(item != 255 for item in assigned[first:point + 1]):
            raise ValueError("overlapping UnicodeData range")
        assigned[first:point + 1] = bytes([kind]) * (point - first + 1)
    if pending is not None:
        raise ValueError("unterminated UnicodeData range")
    return assigned


def verify_domain(actual, defaults, explicit, unicode_data):
    # Independent lookup: choose the smallest containing default, then binary
    # search the explicit sorted interval list. The generation owner instead
    # writes expanded ranges in source precedence order.
    most_specific = sorted(defaults, key=lambda item: item[1] - item[0])
    starts = [first for first, _, _ in explicit]
    assigned = assigned_witness(unicode_data) if unicode_data else None
    for point, observed in enumerate(actual):
        if assigned is not None and assigned[point] != 255:
            expected = assigned[point]
        else:
            expected = next(kind for first, last, kind in most_specific if first <= point <= last)
            index = bisect_right(starts, point) - 1
            if index >= 0 and point <= explicit[index][1]:
                expected = explicit[index][2]
        if observed != expected:
            raise ValueError(f"independent classification mismatch at U+{point:04X}")
    if any(actual[point] != ALIASES["L"] for point in range(0xD800, 0xE000)):
        raise ValueError("surrogate code points must have class L")


def render(actual):
    # L is the complete-domain default. Store only non-L runs; the resulting
    # immutable table requires no allocation or eager runtime expansion.
    ranges, first, previous = [], 0, STRONG.get(actual[0], "Other")
    for point in range(1, DOMAIN + 1):
        current = STRONG.get(actual[point], "Other") if point < DOMAIN else None
        if current != previous:
            if previous != "L":
                ranges.append(f"    (0x{first:X}, 0x{point - 1:X}, BidiClass::{previous}),")
            first, previous = point, current
    table = "\n".join(ranges)
    return HEADER + table + FOOTER


HEADER = '''// Generated by tools/generate_bidi_class.py; do not edit by hand.
// Unicode 18.0.0 DerivedBidiClass data, © 2026 Unicode, Inc.
// Unicode License V3; see the repository Unicode license/notice material.
// UAX #9 revision 52 is external authority; no report text is incorporated.

pub(crate) const UCD_VERSION: &str = "18.0.0";
pub(crate) const UAX9_REVISION: u16 = 52;
pub(crate) const DERIVED_BIDI_CLASS_SHA256: &str =
    "d9e23222522551348ea1ccfbb4f62efbf98982afb95840f8959c08ed992c5607";

/// Only the three strong classes used by HTML first-strong directionality.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BidiClass {
    L,
    R,
    Al,
    Other,
}

/// Accepts Unicode code points, including isolated surrogate code units.
/// Values above the Unicode code-point domain have no property value.
pub(crate) fn bidi_class(code_point: u32) -> Option<BidiClass> {
    if code_point > 0x10FFFF {
        return None;
    }
    let mut left = 0;
    let mut right = NON_L_RANGES.len();
    while left < right {
        let middle = left + (right - left) / 2;
        if NON_L_RANGES[middle].0 <= code_point {
            left = middle + 1;
        } else {
            right = middle;
        }
    }
    let index = left;
    Some(if index > 0 && code_point <= NON_L_RANGES[index - 1].1 {
        NON_L_RANGES[index - 1].2
    } else {
        BidiClass::L
    })
}

static NON_L_RANGES: &[(u32, u32, BidiClass)] = &[
'''

FOOTER = '''
];

#[cfg(test)]
mod tests {
    use super::{BidiClass, DERIVED_BIDI_CLASS_SHA256, UAX9_REVISION, UCD_VERSION, bidi_class};
    use crate::DocumentString;

    #[test]
    fn strong_classes_and_neutral_controls_follow_the_pinned_property() {
        for (point, expected) in [
            (0x41, BidiClass::L),
            (0x5D0, BidiClass::R),
            (0x627, BidiClass::Al),
            (0x30, BidiClass::Other),
            (0x660, BidiClass::Other),
            (0x301, BidiClass::Other),
            (0x202E, BidiClass::Other),
            (0x2067, BidiClass::Other),
            (0x1F600, BidiClass::Other),
        ] {
            assert_eq!(bidi_class(point), Some(expected), "U+{point:04X}");
        }
    }

    #[test]
    fn unassigned_defaults_explicit_exceptions_and_private_use_are_distinct() {
        for (point, expected) in [
            (0x590, BidiClass::R),
            (0x5FF, BidiClass::R),
            (0x7B2, BidiClass::Al),
            (0x20C1, BidiClass::Other),
            (0x378, BidiClass::L),
            (0xE000, BidiClass::L),
            (0xF0000, BidiClass::L),
            (0x10FFFD, BidiClass::L),
            (0xFDD0, BidiClass::Other),
            (0x10FFFF, BidiClass::Other),
            (0xFDDF, BidiClass::Other),
        ] {
            assert_eq!(bidi_class(point), Some(expected), "U+{point:04X}");
        }
    }

    #[test]
    fn unicode_eighteen_assigned_and_reserved_points_use_current_properties() {
        for (point, expected) in [
            (0x10940, BidiClass::R),
            (0x10959, BidiClass::R),
            (0x1095F, BidiClass::R),
            (0x11DB0, BidiClass::L),
            (0x1E6E0, BidiClass::L),
            (0x1E6E3, BidiClass::Other),
        ] {
            assert_eq!(bidi_class(point), Some(expected), "U+{point:04X}");
        }
    }

    #[test]
    fn isolated_surrogates_are_left_to_right_and_pairs_are_single_points() {
        for point in 0xD800..=0xDFFF {
            assert_eq!(bidi_class(point), Some(BidiClass::L));
        }
        let text = DocumentString::from_utf16(vec![0xD800, 0x41, 0xD83D, 0xDE00, 0xDC00]);
        let points: Vec<_> = text.code_points().collect();
        assert_eq!(points, [0xD800, 0x41, 0x1F600, 0xDC00]);
        let classes: Vec<_> = points.into_iter().map(bidi_class).collect();
        assert_eq!(
            classes,
            [
                Some(BidiClass::L),
                Some(BidiClass::L),
                Some(BidiClass::Other),
                Some(BidiClass::L)
            ]
        );
    }

    #[test]
    fn invalid_integers_have_no_unicode_property_value() {
        assert_eq!(bidi_class(0x110000), None);
        assert_eq!(bidi_class(u32::MAX), None);
        assert_eq!(UCD_VERSION, "18.0.0");
        assert_eq!(UAX9_REVISION, 52);
        assert_eq!(
            DERIVED_BIDI_CLASS_SHA256,
            "d9e23222522551348ea1ccfbb4f62efbf98982afb95840f8959c08ed992c5607"
        );
    }

    #[test]
    fn every_code_point_matches_independently_read_normative_records() {
        fn record(raw: &str) -> (u32, u32, BidiClass) {
            let (range, class) = raw.split_once(';').unwrap();
            let range = range.trim();
            let (first, last) = range.split_once("..").unwrap_or((range, range));
            let class = match class.trim() {
                "L" | "Left_To_Right" => BidiClass::L,
                "R" | "Right_To_Left" => BidiClass::R,
                "AL" | "Arabic_Letter" => BidiClass::Al,
                _ => BidiClass::Other,
            };
            (
                u32::from_str_radix(first, 16).unwrap(),
                u32::from_str_radix(last, 16).unwrap(),
                class,
            )
        }
        let mut explicit = Vec::new();
        let mut defaults = Vec::new();
        for line in include_str!("../tools/data/DerivedBidiClass-18.0.0.txt").lines() {
            if let Some((_, raw)) = line.split_once("@missing:") {
                defaults.push(record(raw));
            } else {
                let raw = line.split('#').next().unwrap().trim();
                if !raw.is_empty() {
                    explicit.push(record(raw));
                }
            }
        }
        explicit.sort_unstable_by_key(|&(first, _, _)| first);
        for point in 0..=0x10FFFF {
            let index = explicit.partition_point(|&(first, _, _)| first <= point);
            let expected = if index > 0 && point <= explicit[index - 1].1 {
                explicit[index - 1].2
            } else {
                defaults
                    .iter()
                    .filter(|&&(first, last, _)| first <= point && point <= last)
                    .min_by_key(|&&(first, last, _)| last - first)
                    .unwrap()
                    .2
            };
            assert_eq!(bidi_class(point), Some(expected), "U+{point:04X}");
        }
    }
}
'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--write", action="store_true")
    parser.add_argument("--verify-unicode-data", type=Path)
    args = parser.parse_args()
    defaults, explicit = read_ranges(pinned_text(INPUT, SOURCE_HASH))
    actual = expand(defaults, explicit)
    verify_domain(actual, defaults, explicit, args.verify_unicode_data)
    rendered = render(actual)
    if args.write:
        OUTPUT.write_text(rendered, encoding="utf-8")
    elif OUTPUT.read_text(encoding="utf-8") != rendered:
        raise ValueError(f"{OUTPUT}: generated output is stale; run --write")
    print(f"Bidi_Class {VERSION}: complete {DOMAIN:,}-code-point classification verified; Rust output {'written' if args.write else 'current'}")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
