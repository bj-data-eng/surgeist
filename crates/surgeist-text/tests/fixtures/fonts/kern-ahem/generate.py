#!/usr/bin/env python3
"""Add one deterministic kern pair to retained CC0 Ahem; preserve its full cmap.

Original repository-owned code, using the sfnt assembly mechanics of the owned
PhraseAhem generator. No external font tooling or algorithm is required.
"""

import argparse
import hashlib
from pathlib import Path
import struct


HERE = Path(__file__).resolve().parent
INPUT = HERE.parent / "ahem" / "Ahem.ttf"
OUTPUT = HERE / "KernAhem.ttf"
INPUT_SHA256 = "f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc"
FAMILY = "Surgeist Kern Ahem"
POSTSCRIPT = "SurgeistKernAhem-Regular"
A_GLYPH = 35
PERIOD_GLYPH = 16
KERN_VALUE = -200
IDENTITY_NAMES = {
    1: FAMILY, 2: "Regular", 3: POSTSCRIPT + "-1.000",
    4: FAMILY + " Regular", 5: "Version 1.000", 6: POSTSCRIPT,
    16: FAMILY, 17: "Regular", 18: FAMILY + " Regular",
}


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


def glyph4(subtable, codepoint):
    """Resolve the pinned input's format4 mapping, including range offsets."""
    count = struct.unpack_from(">H", subtable, 6)[0] // 2
    for index in range(count):
        end = struct.unpack_from(">H", subtable, 14 + 2 * index)[0]
        start = struct.unpack_from(">H", subtable, 16 + 2 * count + 2 * index)[0]
        if start <= codepoint <= end:
            delta = struct.unpack_from(">H", subtable, 16 + 4 * count + 2 * index)[0]
            position = 16 + 6 * count + 2 * index
            offset = struct.unpack_from(">H", subtable, position)[0]
            if not offset:
                return (codepoint + delta) & 0xFFFF
            glyph = struct.unpack_from(
                ">H", subtable, position + offset + 2 * (codepoint - start))[0]
            return (glyph + delta) & 0xFFFF if glyph else 0
    return 0


def qualify_cmap(cmap):
    mappings = []
    for index in range(struct.unpack_from(">H", cmap, 2)[0]):
        platform, encoding, offset = struct.unpack_from(">HHI", cmap, 4 + index * 8)
        if platform == 0 or (platform, encoding) == (3, 1):
            subtable = cmap[offset:]
            if struct.unpack_from(">H", subtable)[0] != 4:
                raise ValueError("qualified Ahem Unicode cmap must be format4")
            mappings.append((glyph4(subtable, 0x41), glyph4(subtable, 0x2E)))
    if not mappings or any(pair != (A_GLYPH, PERIOD_GLYPH) for pair in mappings):
        raise ValueError("qualified Ahem A/period mappings differ")


def names(original):
    version, count, offset = struct.unpack_from(">HHH", original)
    if version != 0:
        raise ValueError("qualified Ahem name table must be format0")
    records = []
    for index in range(count):
        platform, encoding, language, name_id, length, start = struct.unpack_from(
            ">6H", original, 6 + index * 12)
        value = original[offset + start:offset + start + length]
        if name_id in IDENTITY_NAMES:
            replacement = IDENTITY_NAMES[name_id]
            if platform in (0, 3):
                value = replacement.encode("utf-16-be")
            elif (platform, encoding) == (1, 0):
                value = replacement.encode("mac_roman")
            else:
                raise ValueError("unsupported qualified Ahem identity encoding")
        # All nonidentity strings, including every upstream legal record, stay
        # byte-identical in their original platform/encoding/language record.
        records.append(((platform, encoding, language, name_id), value))
    records.sort()
    directory = bytearray(struct.pack(">HHH", 0, len(records), 6 + len(records) * 12))
    strings = bytearray()
    for key, value in records:
        directory.extend(struct.pack(">6H", *key, len(value), len(strings)))
        strings.extend(value)
    return bytes(directory + strings)


def kern():
    # OpenType version0, one horizontal format0 subtable; minimum/cross-stream/
    # override flags are absent. Its only sorted pair is actual glyph35/16.
    pair = struct.pack(">HHh", A_GLYPH, PERIOD_GLYPH, KERN_VALUE)
    body = struct.pack(">4H", 1, 6, 0, 0) + pair
    return struct.pack(">2H3H", 0, 1, 0, 6 + len(body), 1) + body


def generate(source):
    if hashlib.sha256(source).hexdigest() != INPUT_SHA256:
        raise ValueError("Ahem input SHA256 differs from the qualified original")
    data = tables(source)
    if b"kern" in data or struct.unpack_from(">H", data[b"head"], 18)[0] != 1000:
        raise ValueError("qualified Ahem table/units differ")
    qualify_cmap(data[b"cmap"])
    data[b"name"] = names(data[b"name"])
    data[b"kern"] = kern()
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
                        help="compare exact bytes without modifying input/output")
    args = parser.parse_args()
    expected = generate(INPUT.read_bytes())
    if args.check:
        if not OUTPUT.exists() or OUTPUT.read_bytes() != expected:
            parser.exit(1, "KernAhem.ttf is missing or stale; run generate.py\n")
    else:
        OUTPUT.write_bytes(expected)
    print(f"{OUTPUT.name}: {hashlib.sha256(expected).hexdigest()}")


if __name__ == "__main__":
    main()
