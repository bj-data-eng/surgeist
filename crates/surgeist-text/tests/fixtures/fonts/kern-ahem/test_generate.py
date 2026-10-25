"""Independent sfnt wire checks for the original standard-library generator."""

import contextlib
import hashlib
import io
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch

import generate


FONT_SHA256 = "51fb4d3bc99025f0a98b2b080101bd9deec0b2aa8a084d3fac91f03e522af4f0"


def directory(font):
    count = int.from_bytes(font[4:6], "big")
    rows = []
    for index in range(count):
        offset = 12 + 16 * index
        tag = font[offset:offset + 4]
        check, start, length = struct.unpack_from(">III", font, offset + 4)
        rows.append((tag, check, start, length))
    return rows


def table_data(font):
    return {tag: font[start:start + length]
            for tag, _, start, length in directory(font)}


def word_sum(data):
    padded = data + bytes(-len(data) % 4)
    return sum(int.from_bytes(padded[i:i + 4], "big")
               for i in range(0, len(padded), 4)) & 0xFFFFFFFF


def unicode_cmaps(data):
    result = {}
    for index in range(int.from_bytes(data[2:4], "big")):
        platform, encoding, start = struct.unpack_from(">HHI", data, 4 + 8 * index)
        if platform != 0 and (platform, encoding) != (3, 1):
            continue
        sub = data[start:]
        if int.from_bytes(sub[:2], "big") != 4:
            raise AssertionError("qualified source has only format4 Unicode cmap")
        count = int.from_bytes(sub[6:8], "big") // 2
        ends = struct.unpack_from(f">{count}H", sub, 14)
        starts = struct.unpack_from(f">{count}H", sub, 16 + 2 * count)
        deltas = struct.unpack_from(f">{count}H", sub, 16 + 4 * count)
        ranges = struct.unpack_from(f">{count}H", sub, 16 + 6 * count)
        mappings = {}
        for segment, (first, last, delta, offset) in enumerate(
                zip(starts, ends, deltas, ranges)):
            for cp in range(first, last + 1):
                if offset:
                    at = 16 + 6 * count + 2 * segment + offset + 2 * (cp - first)
                    glyph = int.from_bytes(sub[at:at + 2], "big")
                    if glyph:
                        glyph = (glyph + delta) & 0xFFFF
                else:
                    glyph = (cp + delta) & 0xFFFF
                if glyph:
                    mappings[cp] = glyph
        result[(platform, encoding)] = mappings
    return result


def name_records(data):
    count, start = struct.unpack_from(">HH", data, 2)
    result = {}
    for index in range(count):
        p, e, language, name_id, size, at = struct.unpack_from(
            ">6H", data, 6 + 12 * index)
        key = p, e, language, name_id
        if key in result:
            raise AssertionError("duplicate name record")
        result[key] = data[start + at:start + at + size]
    return result


class GeneratorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = generate.INPUT.read_bytes()
        cls.font = generate.OUTPUT.read_bytes()

    @classmethod
    def tearDownClass(cls):
        if generate.INPUT.read_bytes() != cls.source:
            raise AssertionError("qualified original input was mutated")

    def test_input_and_legal_bytes_are_exact(self):
        self.assertEqual(hashlib.sha256(self.source).hexdigest(),
                         "f0a92cd0cc45735591c9b5b1fa8aecd5194e8dc518895ca22af94a46c23550dc")
        for filename in ("COPYING", "UPSTREAM-README"):
            self.assertEqual((generate.HERE / filename).read_bytes(),
                             (generate.INPUT.parent / filename).read_bytes())

    def test_reproducible_bytes_and_pure_source(self):
        source = bytearray(self.source)
        self.assertEqual(generate.generate(source), self.font)
        self.assertEqual(source, self.source)
        self.assertEqual(generate.generate(self.source), self.font)
        self.assertEqual(hashlib.sha256(self.font).hexdigest(), FONT_SHA256)

    def test_directory_checksums_alignment_and_zero_padding(self):
        rows = directory(self.font)
        self.assertEqual(self.font[:4], self.source[:4])
        self.assertEqual(len(rows), 12)
        self.assertEqual(self.font[4:12], struct.pack(">4H", 12, 128, 3, 64))
        self.assertEqual([row[0] for row in rows], sorted(row[0] for row in rows))
        self.assertEqual(len({row[0] for row in rows}), 12)
        cursor = 12 + len(rows) * 16
        for tag, check, start, size in rows:
            self.assertEqual(start % 4, 0, tag)
            self.assertGreaterEqual(start, cursor, tag)
            self.assertEqual(self.font[cursor:start], bytes(start - cursor), tag)
            self.assertLessEqual(start + size, len(self.font), tag)
            value = self.font[start:start + size]
            if tag == b"head":
                value = value[:8] + bytes(4) + value[12:]
            self.assertEqual(word_sum(value), check, tag)
            cursor = start + size
        self.assertEqual(self.font[cursor:], bytes(-cursor % 4))
        self.assertEqual(word_sum(self.font), 0xB1B0AFBA)

    def test_original_cmap_outlines_metrics_and_other_tables_preserved(self):
        old, new = table_data(self.source), table_data(self.font)
        self.assertNotIn(b"kern", old)
        self.assertEqual(set(new), set(old) | {b"kern"})
        for tag in set(old) - {b"name", b"head"}:
            self.assertEqual(new[tag], old[tag], tag)
        self.assertEqual(new[b"head"][:8], old[b"head"][:8])
        self.assertEqual(new[b"head"][12:], old[b"head"][12:])

    def test_actual_cmap_pair_and_full_em_metrics(self):
        old, new = table_data(self.source), table_data(self.font)
        mappings = unicode_cmaps(new[b"cmap"])
        self.assertEqual(mappings, unicode_cmaps(old[b"cmap"]))
        self.assertEqual(set(mappings), {(0, 3), (3, 1)})
        for values in mappings.values():
            self.assertEqual(values[0x41], 35)
            self.assertEqual(values[0x2E], 16)
        self.assertEqual(int.from_bytes(new[b"head"][18:20], "big"), 1000)
        count = int.from_bytes(new[b"hhea"][34:36], "big")
        for glyph in (35, 16):
            advance = int.from_bytes(new[b"hmtx"][4 * min(glyph, count - 1):][:2], "big")
            self.assertEqual(advance, 1000)

    def test_exact_one_horizontal_format0_pair_bytes(self):
        tables = table_data(self.font)
        self.assertNotIn(b"GPOS", tables)
        self.assertNotIn(b"kerx", tables)
        data = tables[b"kern"]
        self.assertEqual(data, bytes.fromhex(
            "0000 0001 0000 0014 0001 0001 0006 0000 0000 0023 0010 ff38"))
        version, tables, subversion, length, coverage = struct.unpack_from(">5H", data)
        self.assertEqual((version, tables, subversion, length, coverage),
                         (0, 1, 0, 20, 1))
        self.assertEqual(struct.unpack_from(">4H", data, 10), (1, 6, 0, 0))
        self.assertEqual(struct.unpack_from(">HHh", data, 18), (35, 16, -200))
        self.assertEqual(len(data), 4 + length)

    def test_all_platform_identity_names_and_preserved_other_records(self):
        old = name_records(table_data(self.source)[b"name"])
        new = name_records(table_data(self.font)[b"name"])
        expected = {
            1: "Surgeist Kern Ahem", 2: "Regular",
            3: "SurgeistKernAhem-Regular-1.000", 4: "Surgeist Kern Ahem Regular",
            5: "Version 1.000", 6: "SurgeistKernAhem-Regular",
            16: "Surgeist Kern Ahem", 17: "Regular", 18: "Surgeist Kern Ahem Regular",
        }
        self.assertEqual(set(new), set(old))
        for key, value in new.items():
            platform, encoding, _, name_id = key
            if name_id in expected:
                codec = "mac_roman" if (platform, encoding) == (1, 0) else "utf-16-be"
                self.assertEqual(value.decode(codec), expected[name_id], key)
            else:
                self.assertEqual(value, old[key], key)

    def test_changed_input_is_refused_before_any_output_write(self):
        changed = bytearray(self.source)
        changed[-1] ^= 1
        with self.assertRaisesRegex(ValueError, "SHA256"):
            generate.generate(changed)
        with tempfile.TemporaryDirectory(dir=generate.HERE) as folder:
            source, output = Path(folder) / "changed.ttf", Path(folder) / "output.ttf"
            source.write_bytes(changed)
            output.write_bytes(b"preserve this output")
            with (patch.object(generate, "INPUT", source),
                  patch.object(generate, "OUTPUT", output),
                  patch("sys.argv", ["generate.py"])):
                with self.assertRaisesRegex(ValueError, "SHA256"):
                    generate.main()
            self.assertEqual(source.read_bytes(), changed)
            self.assertEqual(output.read_bytes(), b"preserve this output")

    def test_check_fresh_stale_and_missing_is_nonmutating(self):
        with tempfile.TemporaryDirectory(dir=generate.HERE) as folder:
            output = Path(folder) / "KernAhem.ttf"
            for content in (self.font, b"stale", None):
                if content is None:
                    output.unlink(missing_ok=True)
                else:
                    output.write_bytes(content)
                stamp = output.stat().st_mtime_ns if output.exists() else None
                with (patch.object(generate, "OUTPUT", output),
                      patch("sys.argv", ["generate.py", "--check"]),
                      contextlib.redirect_stdout(io.StringIO()),
                      contextlib.redirect_stderr(io.StringIO())):
                    if content == self.font:
                        generate.main()
                    else:
                        with self.assertRaises(SystemExit) as raised:
                            generate.main()
                        self.assertEqual(raised.exception.code, 1)
                if content is None:
                    self.assertFalse(output.exists())
                else:
                    self.assertEqual(output.read_bytes(), content)
                    self.assertEqual(output.stat().st_mtime_ns, stamp)

    def test_generation_cli_writes_only_expected_font(self):
        with tempfile.TemporaryDirectory(dir=generate.HERE) as folder:
            output = Path(folder) / "KernAhem.ttf"
            with (patch.object(generate, "OUTPUT", output),
                  patch("sys.argv", ["generate.py"]),
                  contextlib.redirect_stdout(io.StringIO())):
                generate.main()
            self.assertEqual(output.read_bytes(), self.font)
            self.assertEqual(list(Path(folder).iterdir()), [output])


if __name__ == "__main__":
    unittest.main()
