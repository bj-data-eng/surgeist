#!/usr/bin/env python3
"""Generate ordinary HTML named-reference/C1 data, offline and without packages.

--check verifies the output; --write regenerates it. Both derive every record
from the tracked complete tables in the retained normative syntax capture.
--verify-origins additionally checks the full pinned HTML source and the acquired
html5ever entities.rs/lib.rs witnesses. No engine, atom, or provider code runs.

The incorporated WHATWG table data is BSD-3-Clause; see licenses/whatwg/.
Engine files are independent comparison witnesses, not executable generation
dependencies or imported atom implementation.
"""

import argparse
import hashlib
from html.parser import HTMLParser
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[3]
REVISION = "877d6146487f15b15bc8dd0e3c625063a37b456a"
ENGINE_REVISION = "d7232d746d3112f08926d169591576bd12fe2fdf"
FULL_HASH = "877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f"
CAPTURE_HASH = "db1eaacc388d494736080301dd18e9147feb39f2651e2bc7fbeed36ee2cfb214"
ENTITIES_HASH = "4f1f33f9d1c8777371be06e2fd2987d7721b1229daeb4a0879f3f800d50256c9"
LIBRARY_HASH = "c09bff419cb84f51a5d43b6cb00b9a07af12b472ff41a9ab4b347e316c1f98ef"
CAPTURE = ROOT / "references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md"
FULL_SOURCE = ROOT / "tmp/work/html-contracts/sources/html--877d6146487f--complete.html"
ENGINE_DIRECTORY = ROOT / "tmp/work/html-parser-suitability/html5ever/web_atoms"
OUTPUT = ROOT / "crates/surgeist-html/src/parser/character_references.rs"


def read_pinned(path, expected):
    data = path.read_bytes()
    actual = hashlib.sha256(data).hexdigest()
    if actual != expected:
        raise ValueError(f"{path}: expected SHA-256 {expected}, got {actual}")
    return data.decode("utf-8")


def scalar(value):
    return 0 <= value <= 0x10FFFF and not 0xD800 <= value <= 0xDFFF


class ReferenceTables(HTMLParser):
    """Read table cells independently, including omitted HTML end tags."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.table = None
        self.kind = None
        self.cells = []
        self.cell = None
        self.names = {}
        self.overrides = {}

    def finish_cell(self):
        if self.cell is not None:
            self.cells.append("".join(self.cell).strip())
            self.cell = None

    def finish_row(self):
        self.finish_cell()
        if self.kind == "named":
            if len(self.cells) != 3:
                raise ValueError(f"named-reference row has {len(self.cells)} cells")
            name = self.cells[0]
            if not re.fullmatch(r"[A-Za-z0-9]+;?", name):
                raise ValueError(f"unexpected reference name {name!r}")
            points = tuple(int(value, 16) for value in re.findall(r"U\+([0-9A-F]+)", self.cells[1]))
            if not 1 <= len(points) <= 2 or not all(scalar(point) for point in points):
                raise ValueError(f"invalid reference scalars for {name!r}: {points}")
            if name in self.names:
                raise ValueError(f"duplicate reference {name!r}")
            self.names[name] = points
        elif self.kind == "c1" and self.cells:
            if len(self.cells) != 3:
                raise ValueError("C1 override row has unexpected cell boundaries")
            source, target = self.cells[:2]
            if not re.fullmatch(r"0x[0-9A-Fa-f]+", source) or not re.fullmatch(r"0x[0-9A-Fa-f]+", target):
                raise ValueError(f"unexpected C1 row {self.cells}")
            source, target = int(source, 16), int(target, 16)
            if not 0x80 <= source <= 0x9F or not scalar(target):
                raise ValueError("C1 override outside its scalar domain")
            if source in self.overrides:
                raise ValueError(f"duplicate C1 override {source:#x}")
            self.overrides[source] = target
        self.kind = None
        self.cells = []

    def handle_starttag(self, tag, attrs):
        attributes = dict(attrs)
        if tag == "table":
            self.finish_row()
            self.table = attributes.get("id")
        elif tag == "tr":
            self.finish_row()
            identity = attributes.get("id", "")
            if identity.startswith("entity-"):
                self.kind = "named"
            elif self.table == "table-charref-overrides":
                self.kind = "c1"
        elif tag == "th":
            # Column headers are not data rows.
            self.kind = None
        elif tag == "td" and self.kind is not None:
            self.finish_cell()
            self.cell = []

    def handle_data(self, data):
        if self.cell is not None:
            self.cell.append(data)

    def handle_endtag(self, tag):
        if tag == "td":
            self.finish_cell()
        elif tag == "tr":
            self.finish_row()
        elif tag == "table":
            self.finish_row()
            self.table = None

    def records(self, text):
        self.feed(text)
        self.close()
        self.finish_row()
        if not self.names or not self.overrides:
            raise ValueError("complete named-reference/C1 tables not found")
        return self.names, self.overrides


def equal_records(label, expected, actual):
    missing = sorted(set(expected) - set(actual))
    extra = sorted(set(actual) - set(expected))
    changed = sorted(key for key in set(expected) & set(actual) if expected[key] != actual[key])
    if missing or extra or changed:
        raise ValueError(f"{label}: missing={missing[:5]}, extra={extra[:5]}, changed={changed[:5]}")


def verify_origins(names, overrides, html_source, engine_directory):
    full = ReferenceTables().records(read_pinned(html_source, FULL_HASH))
    equal_records("full normative names", names, full[0])
    equal_records("full normative C1 overrides", overrides, full[1])
    engine_text = read_pinned(engine_directory / "entities.rs", ENTITIES_HASH)
    engine = {}
    for name, first, second in re.findall(r'\("&([A-Za-z0-9]+;?)", (\d+), (\d+)\)', engine_text):
        if name in engine:
            raise ValueError(f"duplicate engine reference {name!r}")
        first, second = int(first), int(second)
        engine[name] = (first, second) if second else (first,)
    equal_records("html5ever named mappings", names, engine)
    library = read_pinned(engine_directory / "lib.rs", LIBRARY_HASH)
    match = re.search(r"C1_REPLACEMENTS: \[Option<char>; 32\] = \[(.*?)\];", library, re.S)
    if match is None:
        raise ValueError("engine C1 witness not found")
    fields = re.findall(r"Some\('\\u\{([0-9a-fA-F]+)\}'\)|\b(None)\b", match.group(1))
    if len(fields) != 32:
        raise ValueError(f"engine C1 witness has {len(fields)} entries")
    engine_c1 = {0x80 + index: int(value, 16) for index, (value, absent) in enumerate(fields) if not absent}
    equal_records("html5ever C1 mappings", overrides, engine_c1)


def rust_char(point):
    return "'\\u{" + f"{point:x}" + "}'"


def render(names, overrides):
    lines = [
        "// Generated by tools/generate_character_references.py; do not edit.",
        f"// Normative HTML revision: {REVISION}",
        f"// Complete normative HTML SHA-256: {FULL_HASH}",
        f"// Retained complete syntax-table capture SHA-256: {CAPTURE_HASH}",
        f"// Independently cross-verified html5ever revision: {ENGINE_REVISION}",
        f"// web_atoms/entities.rs SHA-256: {ENTITIES_HASH}",
        f"// web_atoms/lib.rs (C1 witness) SHA-256: {LIBRARY_HASH}",
        "// Ordinary WHATWG table data only; no atom/provider implementation.",
        "// WHATWG table copyright and BSD-3-Clause terms: licenses/whatwg/.",
        "",
        "use super::ReferenceValue;",
        "",
        "#[rustfmt::skip]",
        f"pub(super) static NAMED_REFERENCES: [(&str, ReferenceValue); {len(names)}] = [",
    ]
    for name, points in sorted(names.items()):
        arguments = ", ".join(rust_char(point) for point in points)
        variant = "One" if len(points) == 1 else "Two"
        lines.append(f'    ("{name}", ReferenceValue::{variant}({arguments})),')
    lines += ["];","", "#[rustfmt::skip]", "pub(super) const C1_REPLACEMENTS: [Option<char>; 32] = ["]
    for code in range(0x80, 0xA0):
        lines.append(f"    Some({rust_char(overrides[code])})," if code in overrides else "    None,")
    lines += ["];",""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--check", action="store_true")
    action.add_argument("--write", action="store_true")
    parser.add_argument("--verify-origins", action="store_true")
    parser.add_argument("--html-source", type=Path, default=FULL_SOURCE)
    parser.add_argument("--engine-directory", type=Path, default=ENGINE_DIRECTORY)
    args = parser.parse_args()
    try:
        names, overrides = ReferenceTables().records(read_pinned(CAPTURE, CAPTURE_HASH))
        if args.verify_origins:
            verify_origins(names, overrides, args.html_source, args.engine_directory)
        expected = render(names, overrides)
        if args.write:
            OUTPUT.write_text(expected, encoding="utf-8", newline="\n")
        elif not OUTPUT.is_file() or OUTPUT.read_bytes() != expected.encode("utf-8"):
            raise ValueError(f"{OUTPUT}: generated output differs; run --write")
        print(f"verified {len(names)} complete names, {sum(len(points) == 2 for points in names.values())} two-scalar mappings and {len(overrides)} C1 overrides" + (" against both pinned originals" if args.verify_origins else " from retained normative tables"))
    except (OSError, UnicodeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
