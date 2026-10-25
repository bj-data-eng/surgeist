"""Behavioral checks for the original stdlib-only fixture generator."""

import contextlib
import hashlib
import io
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch

import generate


class GeneratorTests(unittest.TestCase):
    def test_exact_input_and_preserved_legal_bytes(self):
        self.assertEqual(hashlib.sha256(generate.INPUT.read_bytes()).hexdigest(),
                         generate.INPUT_SHA256)
        for filename in ("COPYING", "UPSTREAM-README"):
            self.assertEqual((generate.HERE / filename).read_bytes(),
                             (generate.HERE.parent / "ahem" / filename).read_bytes())

    def test_deterministic_font_and_checksums_preserve_outline(self):
        source = generate.INPUT.read_bytes()
        font = generate.generate(source)
        self.assertEqual(font, generate.generate(source))
        self.assertEqual(font, generate.OUTPUT.read_bytes())
        self.assertEqual(generate.checksum(font), 0xB1B0AFBA)
        old, new = generate.tables(source), generate.tables(font)
        self.assertEqual(set(old), set(new))
        for tag in set(old) - {b"head", b"cmap", b"name", b"OS/2"}:
            self.assertEqual(old[tag], new[tag], tag)
        count = struct.unpack_from(">H", font, 4)[0]
        for index in range(count):
            tag, expected, offset, length = struct.unpack_from(">4sIII", font, 12 + index * 16)
            self.assertEqual(offset % 4, 0)
            data = font[offset:offset + length]
            if tag == b"head":
                data = data[:8] + bytes(4) + data[12:]
            self.assertEqual(generate.checksum(data), expected, tag)

    def test_cmap_has_only_the_declared_nine_mappings_and_own_family(self):
        data = generate.tables(generate.generate(generate.INPUT.read_bytes()))
        cmap = data[b"cmap"]
        for index, platform in enumerate([(0, 3), (3, 1)]):
            actual_platform, encoding, offset = struct.unpack_from(">HHI", cmap, 4 + index * 8)
            self.assertEqual((actual_platform, encoding), platform)
            sub = cmap[offset:]
            count = struct.unpack_from(">H", sub, 6)[0] // 2
            self.assertEqual(struct.unpack_from(">H", sub)[0], 4)
            ends = struct.unpack_from(f">{count}H", sub, 14)
            starts = struct.unpack_from(f">{count}H", sub, 16 + count * 2)
            deltas = struct.unpack_from(f">{count}H", sub, 16 + count * 4)
            ranges = struct.unpack_from(f">{count}H", sub, 16 + count * 6)
            self.assertEqual(ends, starts)
            self.assertEqual(set(ranges), {0})
            mappings = {code: (code + delta) & 0xFFFF
                        for code, delta in zip(starts[:-1], deltas[:-1])}
            self.assertEqual(mappings, generate.MAPPINGS)
        name = data[b"name"]
        count, offset = struct.unpack_from(">HH", name, 2)
        strings = {}
        for index in range(count):
            platform, encoding, language, key, length, start = struct.unpack_from(
                ">6H", name, 6 + index * 12)
            if (platform, encoding, language) == (3, 1, 0x0409):
                strings[key] = name[offset + start:offset + start + length].decode("utf-16-be")
        self.assertEqual(strings[1], generate.FAMILY)
        self.assertEqual(strings[16], generate.FAMILY)
        self.assertEqual(strings[6], "SurgeistPhraseAhem-Regular")

    def test_changed_input_is_refused(self):
        source = bytearray(generate.INPUT.read_bytes())
        source[-1] ^= 1
        with self.assertRaisesRegex(ValueError, "SHA256"):
            generate.generate(source)

    def test_check_is_nonmutating_for_fresh_and_stale_files(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "PhraseAhem.ttf"
            fresh = generate.generate(generate.INPUT.read_bytes())
            for content, expected_exit in [(fresh, None), (b"stale", 1)]:
                output.write_bytes(content)
                with (patch.object(generate, "OUTPUT", output),
                      patch("sys.argv", ["generate.py", "--check"]),
                      contextlib.redirect_stdout(io.StringIO()),
                      contextlib.redirect_stderr(io.StringIO())):
                    if expected_exit is None:
                        generate.main()
                    else:
                        with self.assertRaises(SystemExit) as raised:
                            generate.main()
                        self.assertEqual(raised.exception.code, expected_exit)
                self.assertEqual(output.read_bytes(), content)


if __name__ == "__main__":
    unittest.main()
