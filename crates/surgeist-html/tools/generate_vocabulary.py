#!/usr/bin/env python3
"""Generate the ordinary Rust vocabulary inventory from retained HTML tables.

Run from any directory: python3 crates/surgeist-html/tools/generate_vocabulary.py
--check (read-only) or --write. This offline, standard-library-only generator owns
the marked region in src/vocabulary.rs. It parses the original semantic tables,
expands the heading group, and retains contextual rows without deriving complete
semantics from the non-normative index. No Cargo build-time generation occurs.

Input portions incorporated into code are copyright WHATWG (Apple, Google,
Mozilla, Microsoft), BSD 3-Clause under the exact licenses/whatwg/LICENSE. See
licenses/whatwg/NOTICE.md for edition identity and source adaptations.
"""

import argparse
import hashlib
from html.parser import HTMLParser
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[3]
CAPTURE = "references/html-language-elements-attributes--877d6146487f--2ca83b5ac984.md"
REVISION = "877d6146487f15b15bc8dd0e3c625063a37b456a"
ORIGINAL_SHA256 = "877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f"
SNAPSHOT = f"https://html.spec.whatwg.org/commit-snapshots/{REVISION}/"
OUTPUT = ROOT / "crates/surgeist-html/src/vocabulary.rs"
BEGIN = "// BEGIN GENERATED VOCABULARY\n"
END = "// END GENERATED VOCABULARY"


class IndexTables(HTMLParser):
    """Preserve cell boundaries and link order instead of flattening HTML rows."""

    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.tables = []
        self.table = None
        self.row = None
        self.cell = None
        self.link = None
        self.in_caption = False

    def handle_starttag(self, tag, attrs):
        if tag == "table":
            if self.table is not None:
                raise ValueError("nested table requires an explicit source conversion")
            self.table = {"caption": "", "rows": []}
        elif tag == "caption" and self.table is not None:
            self.in_caption = True
        elif tag == "tr" and self.table is not None:
            self.row = []
        elif tag in ("td", "th") and self.row is not None:
            attributes = dict(attrs)
            if attributes.get("rowspan", "1") != "1" or attributes.get("colspan", "1") != "1":
                raise ValueError("spanning index cell requires explicit expansion")
            self.cell = {"text": "", "links": []}
        elif tag == "a" and self.cell is not None:
            self.link = dict(attrs).get("href")
            if self.link:
                self.cell["links"].append(self.link)

    def handle_data(self, data):
        if self.in_caption:
            self.table["caption"] += data
        if self.cell is not None:
            self.cell["text"] += data

    def handle_endtag(self, tag):
        if tag == "a":
            self.link = None
        elif tag == "caption":
            self.in_caption = False
        elif tag in ("td", "th") and self.cell is not None:
            self.cell["text"] = " ".join(self.cell["text"].split())
            self.row.append(self.cell)
            self.cell = None
        elif tag == "tr" and self.row is not None:
            self.table["rows"].append(self.row)
            self.row = None
        elif tag == "table" and self.table is not None:
            self.tables.append(self.table)
            self.table = None


def rust_string(value):
    result = '"'
    for char in value:
        if char in ('"', "\\"):
            result += "\\" + char
        elif ord(char) < 32:
            result += f"\\u{{{ord(char):x}}}"
        else:
            result += char
    return result + '"'


def variant(name):
    return "".join(part[0].upper() + part[1:] for part in name.split("-"))


def expanded_reference(reference):
    if reference.startswith("#"):
        return SNAPSHOT + reference
    if reference.startswith("html-language-") and ".md#" in reference:
        return SNAPSHOT + "#" + reference.split(".md#", 1)[1]
    return reference


def references(cells):
    return list(dict.fromkeys(expanded_reference(href) for cell in cells for href in cell["links"]))


def array(values):
    return "&[" + ", ".join(values) + "]"


def enum_definition(name, namespace, names, description):
    values = "\n".join(f"        {variant(local)} => {rust_string(local)}," for local in names)
    return f'''#[rustfmt::skip]
vocabulary! {{
    /// {description}
    {name}, {namespace}, [
{values}
    ]
}}
'''


def table(parser, caption, columns, count):
    found = [item for item in parser.tables if " ".join(item["caption"].split()) == caption]
    if len(found) != 1:
        raise ValueError(f"expected one {caption!r} table, found {len(found)}")
    rows = found[0]["rows"]
    if len(rows) != count + 1 or any(len(row) != columns for row in rows):
        raise ValueError(f"{caption}: changed table shape or inventory count")
    return rows[1:]


def generate():
    raw = (ROOT / CAPTURE).read_bytes()
    text = raw.decode("utf-8")
    if f"Source revision `{REVISION}`" not in text or ORIGINAL_SHA256 not in text:
        raise ValueError("retained input does not identify the selected HTML edition")
    parser = IndexTables()
    # The retained complex tables remain literal semantic HTML on single lines.
    # Feed those tables only; Markdown prose is not an HTML document.
    for line in text.splitlines():
        if line.startswith("<table"):
            parser.feed(line)
    elements = table(parser, "List of elements", 7, 111)
    attributes = table(parser, "List of attributes (excluding event handler content attributes)", 4, 180)

    element_names = []
    row_names = []
    open_rows = {"MathML math", "SVG svg", "autonomous custom elements"}
    for row in elements:
        printed = row[0]["text"]
        names = [] if printed in open_rows else printed.split(", ")
        if any(not re.fullmatch(r"[a-z][a-z0-9]*", name) for name in names):
            raise ValueError(f"element family requires explicit handling: {printed}")
        row_names.append(names)
        element_names.extend(names)
    attribute_names = list(dict.fromkeys(row[0]["text"] for row in attributes))
    if len(element_names) != 113 or len(set(element_names)) != 113 or len(attribute_names) != 144:
        raise ValueError("changed expanded element or distinct attribute identity count")
    if len({variant(name) for name in attribute_names}) != len(attribute_names):
        raise ValueError("attribute enum variant collision")

    output = "// Generated by tools/generate_vocabulary.py from retained non-normative indexes.\n\n"
    output += enum_definition("KnownHtmlElement", "html", element_names,
                              "Individually named HTML elements in the pinned modern element index.")
    output += "\n" + enum_definition("KnownHtmlAttribute", "none", attribute_names,
                                    "Local names in the pinned ordinary attribute index; contextual rows remain separate.")
    output += f'''
/// Immutable source identity for these generated inventory records.
pub const VOCABULARY_SOURCE: VocabularySource = VocabularySource {{
    revision: {rust_string(REVISION)},
    original_html_sha256: {rust_string(ORIGINAL_SHA256)},
    retained_capture: {rust_string(CAPTURE)},
    retained_capture_sha256: {rust_string(hashlib.sha256(raw).hexdigest())},
    elements_url: {rust_string(SNAPSHOT + "#elements-3")},
    attributes_url: {rust_string(SNAPSHOT + "#attributes-3")},
}};

#[rustfmt::skip]
static ELEMENT_INVENTORY: &[ElementInventoryRow] = &[
'''
    for number, (row, names) in enumerate(zip(elements, row_names), 1):
        values = [cell["text"] for cell in row]
        output += "    ElementInventoryRow {\n"
        output += f"        row: {number},\n        name: {rust_string(values[0])},\n"
        output += "        html_elements: " + array(f"KnownHtmlElement::{variant(name)}" for name in names) + ",\n"
        for field, value in zip(("description", "categories", "parents", "children", "attributes", "interface"), values[1:]):
            output += f"        {field}: {rust_string(value)},\n"
        output += "        references: " + array(rust_string(ref) for ref in references(row[:1])) + ",\n    },\n"
    output += "];\n\n#[rustfmt::skip]\nstatic ATTRIBUTE_INVENTORY: &[AttributeInventoryRow] = &[\n"
    known = set(element_names)
    for number, row in enumerate(attributes, 1):
        values = [cell["text"] for cell in row]
        printed = values[1]
        # Parent qualifications are not applicable element names. Form-associated
        # custom elements are an open registered family, not the built-in form.
        names_text = re.sub(r"\([^)]*\)", "", printed)
        names_text = names_text.replace("form-associated custom elements", "").replace("HTML elements", "")
        names = list(dict.fromkeys(re.findall(r"[a-z][a-z0-9-]*", names_text)))
        if any(name not in known for name in names):
            raise ValueError(f"unhandled attribute applicability: {printed!r}")
        output += "    AttributeInventoryRow {\n"
        output += f"        row: {number},\n        attribute: KnownHtmlAttribute::{variant(values[0])},\n"
        output += f"        applicability: {rust_string(printed)},\n"
        output += "        html_elements: " + array(f"KnownHtmlElement::{variant(name)}" for name in names) + ",\n"
        output += f"        all_html_elements: {str('HTML elements' in printed).lower()},\n"
        output += f"        form_associated_custom_elements: {str('form-associated custom elements' in printed).lower()},\n"
        output += f"        description: {rust_string(values[2])},\n        value_syntax: {rust_string(values[3])},\n"
        output += "        references: " + array(rust_string(ref) for ref in references(row)) + ",\n    },\n"
    return output + "];\n"


def main():
    arguments = argparse.ArgumentParser(description=__doc__)
    mode = arguments.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="compare generated region without writing")
    mode.add_argument("--write", action="store_true", help="replace the owned generated region")
    options = arguments.parse_args()
    current = OUTPUT.read_text()
    if current.count(BEGIN) != 1 or current.count(END) != 1:
        raise ValueError("generated region markers missing or duplicated")
    before, rest = current.split(BEGIN, 1)
    _, after = rest.split(END, 1)
    expected = before + BEGIN + generate() + END + after
    if options.check:
        if current != expected:
            print("HTML vocabulary inventory differs; regenerate with --write", file=sys.stderr)
            return 1
        print("HTML vocabulary inventory matches retained source (111 rows / 113 HTML names; 180 rows / 144 attribute names)")
    else:
        OUTPUT.write_text(expected)
        print("generated HTML vocabulary inventory")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
