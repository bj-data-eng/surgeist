#!/usr/bin/env python3
"""Derive a small, deterministic phrase-test cmap from the bundled CC0 Ahem."""

import argparse
import hashlib
from pathlib import Path
import struct


HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / "ahem" / "Ahem.ttf"
OUTPUT = HERE / "PhraseAhem.ttf"
INPUT_SHA256 = "f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc"
FAMILY = "Surgeist Phrase Ahem"
# Ahem's actual square outline, not its missing-glyph outline. SPACE and A
# retain their original mappings. No other character coverage is advertised.
MAPPINGS = {0x20: 3, 0x41: 35, **{ord(c): 35 for c in "日本へようこそ"}}


def checksum(data):
    padded = data + bytes(-len(data) % 4)
    return sum(struct.unpack(f">{len(padded) // 4}I", padded)) & 0xFFFFFFFF


def tables(font):
    count = struct.unpack_from(">H", font, 4)[0]
    result = {}
    for index in range(count):
        tag, _, offset, length = struct.unpack_from(">4sIII", font, 12 + index * 16)
        if tag in result or offset + length > len(font):
            raise ValueError("invalid or duplicate input table")
        result[tag] = font[offset:offset + length]
    return result


def cmap():
    # Format 4 singleton segments work for all nine BMP mappings. The sentinel
    # maps U+FFFF to glyph0. Both platform records share the same subtable.
    pairs = sorted(MAPPINGS.items())
    codes = [code for code, _ in pairs] + [0xFFFF]
    deltas = [(glyph - code) & 0xFFFF for code, glyph in pairs] + [1]
    count = len(codes)
    power = 1 << (count.bit_length() - 1)
    subtable = struct.pack(">7H", 4, 16 + count * 8, 0, count * 2,
                           power * 2, power.bit_length() - 1,
                           count * 2 - power * 2)
    subtable += struct.pack(f">{count}H", *codes) + bytes(2)
    subtable += struct.pack(f">{count}H", *codes)
    subtable += struct.pack(f">{count}H", *deltas) + bytes(count * 2)
    return (struct.pack(">HH", 0, 2)
            + struct.pack(">HHI", 0, 3, 20)
            + struct.pack(">HHI", 3, 1, 20) + subtable)


def names(original):
    count, offset = struct.unpack_from(">HH", original, 2)
    # Preserve upstream legal strings verbatim, including original encodings.
    records = []
    for index in range(count):
        platform, encoding, language, name_id, length, start = struct.unpack_from(
            ">6H", original, 6 + index * 12)
        if name_id in (0, 7, 8, 9, 11, 12, 13, 14):
            records.append(((platform, encoding, language, name_id),
                            original[offset + start:offset + start + length]))
    replacements = {
        1: FAMILY, 2: "Regular", 3: "SurgeistPhraseAhem-Regular-1.000",
        4: FAMILY + " Regular", 5: "Version 1.000",
        6: "SurgeistPhraseAhem-Regular", 16: FAMILY, 17: "Regular",
    }
    records += [((3, 1, 0x0409, key), value.encode("utf-16-be"))
                for key, value in replacements.items()]
    records.sort()
    directory = bytearray(struct.pack(">HHH", 0, len(records), 6 + len(records) * 12))
    strings = bytearray()
    for key, value in records:
        directory.extend(struct.pack(">6H", *key, len(value), len(strings)))
        strings.extend(value)
    return bytes(directory + strings)


def generate(source):
    if hashlib.sha256(source).hexdigest() != INPUT_SHA256:
        raise ValueError("Ahem input SHA256 differs from the qualified original")
    data = tables(source)
    data[b"cmap"] = cmap()
    data[b"name"] = names(data[b"name"])
    # OS/2 range metadata matches the deliberately narrow new cmap: Basic Latin,
    # Hiragana and CJK Unified Ideographs, and Latin/Japanese code pages.
    os2 = bytearray(data[b"OS/2"])
    struct.pack_into(">4I", os2, 42, 1, (1 << (49 - 32)) | (1 << (59 - 32)), 0, 0)
    struct.pack_into(">HH", os2, 64, min(MAPPINGS), max(MAPPINGS))
    if struct.unpack_from(">H", os2)[0] >= 1:
        struct.pack_into(">2I", os2, 78, (1 << 0) | (1 << 17), 0)
    data[b"OS/2"] = bytes(os2)
    count = len(data)
    power = 1 << (count.bit_length() - 1)
    output = bytearray(struct.pack(">4s4H", source[:4], count, power * 16,
                                   power.bit_length() - 1, (count - power) * 16))
    output.extend(bytes(count * 16))
    head_offset = None
    for index, (tag, value) in enumerate(sorted(data.items())):
        if tag == b"head":
            value = value[:8] + bytes(4) + value[12:]
            head_offset = len(output)
        struct.pack_into(">4sIII", output, 12 + index * 16,
                         tag, checksum(value), len(output), len(value))
        output.extend(value)
        output.extend(bytes(-len(output) % 4))
    assert head_offset is not None
    struct.pack_into(">I", output, head_offset + 8,
                     (0xB1B0AFBA - checksum(output)) & 0xFFFFFFFF)
    assert checksum(output) == 0xB1B0AFBA
    return bytes(output)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true",
                        help="compare exact bytes without modifying the fixture")
    args = parser.parse_args()
    expected = generate(INPUT.read_bytes())
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_bytes() != expected:
            parser.exit(1, "PhraseAhem.ttf is missing or stale; run generate.py\n")
    else:
        OUTPUT.write_bytes(expected)
    print(f"{OUTPUT.name}: {hashlib.sha256(expected).hexdigest()}")


if __name__ == "__main__":
    main()
