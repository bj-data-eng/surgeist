"""Behavioral schema and exact-artifact checks for the offline phrase generator."""

from contextlib import redirect_stderr, redirect_stdout
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import generate


class GeneratorTests(unittest.TestCase):
    def setUp(self):
        self.raw = generate.INPUT.read_bytes()
        self.model = generate.pinned_model(self.raw)

    def test_qualified_asset_identities(self):
        for relative, length, digest in [
            ("data/budoux/0.9.3/ja.json", 20190, generate.SOURCE_SHA256),
            ("data/budoux/0.9.3/ja.tsv", 2772,
             "eb7ea47fb591447ac7bc0d01ec0e01be58da9903865d032b76b4aa72352e4a01"),
            ("licenses/budoux/LICENSE", 11357,
             "58d1e17ffe5109a7ae296caafcadfdbe6a7d176f0bc4ab01e12a689b0499d8bd"),
        ]:
            with self.subTest(path=relative):
                raw = (generate.CRATE / relative).read_bytes()
                self.assertEqual(len(raw), length)
                self.assertEqual(hashlib.sha256(raw).hexdigest(), digest)

    def test_deterministic_output_ignores_json_member_order(self):
        reordered = {name: dict(reversed(list(entries.items())))
                     for name, entries in reversed(list(self.model.items()))}
        self.assertEqual(generate.render(self.model), generate.render(reordered))
        self.assertEqual(generate.render(self.model), generate.OUTPUT.read_bytes())

    def test_schema_refuses_duplicates_and_non_json_numbers(self):
        for raw in ['{"UW1":{},"UW1":{}}', '{"UW1":{"A":1,"A":2}}',
                    '{"UW1":{"A":NaN}}', '{"UW1":{"A":Infinity}}']:
            with self.subTest(raw=raw), self.assertRaises(ValueError):
                generate.load_model(raw)

    def test_schema_refuses_wrong_groups_keys_and_weights(self):
        alterations = [
            lambda m: m.update({"OTHER": {"A": 1}}),
            lambda m: m.pop("UW1"),
            lambda m: m.update({"UW1": []}),
            lambda m: m.update({"UW1": {}}),
            lambda m: m["UW1"].update({"AB": 1}),
            lambda m: m["UW1"].update({"\ud800": 1}),
            lambda m: m["UW1"].update({"😀": 1}),
            lambda m: m["UW1"].update({"A": True}),
            lambda m: m["UW1"].update({"A": 0.5}),
            lambda m: m["UW1"].update({"A": "1"}),
            lambda m: m["UW1"].update({"A": 32768}),
            lambda m: m["UW1"].update({"A": -32769}),
        ]
        for index, alter in enumerate(alterations):
            model = json.loads(self.raw)
            alter(model)
            with self.subTest(index=index), self.assertRaises(ValueError):
                generate.load_model(json.dumps(model))

    def test_pin_refuses_changed_bytes_even_when_semantically_equivalent(self):
        with self.assertRaises(ValueError):
            generate.pinned_model(self.raw + b"\n")

    def test_check_does_not_rewrite_current_or_stale_output(self):
        for current in [True, False]:
            with self.subTest(current=current), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / "phrase_data.rs"
                output.write_bytes(generate.render(self.model) if current else b"stale\n")
                before = output.read_bytes(), output.stat().st_mtime_ns
                source_before = generate.INPUT.read_bytes(), generate.INPUT.stat().st_mtime_ns
                with patch.object(generate, "OUTPUT", output), redirect_stdout(io.StringIO()), \
                        redirect_stderr(io.StringIO()):
                    self.assertEqual(generate.main(["--check"]), 0 if current else 1)
                self.assertEqual((output.read_bytes(), output.stat().st_mtime_ns), before)
                self.assertEqual((generate.INPUT.read_bytes(), generate.INPUT.stat().st_mtime_ns),
                                 source_before)


if __name__ == "__main__":
    unittest.main()
