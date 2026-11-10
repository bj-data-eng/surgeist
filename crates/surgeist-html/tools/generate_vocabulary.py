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

The generator also owns the perfect hashes over UTF-16 code units that recognize the
closed element and event handler names, and the foreign element names that the tree
construction algorithm tests, taken from the retained syntax capture of the same
edition. The hash is a hash-and-displace scheme with a seed found by a deterministic
search, so the output is reproducible. Its lookup is in src/internal/values/name_hash.rs.
"""

import argparse
import hashlib
import html
from html.parser import HTMLParser
from pathlib import Path
import re
import sys


ROOT = Path(__file__).resolve().parents[3]
CAPTURE = "references/html-language-elements-attributes--877d6146487f--2ca83b5ac984.md"
REVISION = "877d6146487f15b15bc8dd0e3c625063a37b456a"
ORIGINAL_SHA256 = "877d6cc00dae6c18c16d952fdcbf944acc58919579621b4257fa0ed1888a871f"
SYNTAX_CAPTURE = "references/html-language-syntax-parser--877d6146487f--37a9ba51eed8.md"
SYNTAX_CAPTURE_SHA256 = "db1eaacc388d494736080301dd18e9147feb39f2651e2bc7fbeed36ee2cfb214"
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


def enum_definition(name, namespace, names, description, hash=None):
    if len({variant(local) for local in names}) != len(names):
        raise ValueError(f"{name}: enum variant collision")
    values = "\n".join(f"        {variant(local)} => {rust_string(local)}," for local in names)
    hash_argument = f"{hash}, " if hash else ""
    return f'''#[rustfmt::skip]
vocabulary! {{
    /// {description}
    {name}, {namespace}, {hash_argument}[
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

MASK64 = (1 << 64) - 1


def hash_units(units, seed):
    """The 64-bit hash of UTF-16 code units; src/internal/values/name_hash.rs computes the same."""
    state = (seed ^ 0xCBF29CE484222325) & MASK64
    for unit in units:
        state = ((state ^ unit) * 0x100000001B3) & MASK64
    state ^= state >> 30
    state = (state * 0xBF58476D1CE4E5B9) & MASK64
    state ^= state >> 27
    state = (state * 0x94D049BB133111EB) & MASK64
    state ^= state >> 31
    return state


def hash_parts(state):
    """The bucket selector and the two displacement inputs of a hash."""
    selector = ((state << 21) | (state >> 43)) & 0xFFFFFFFF
    return selector, state & 0xFFFFFFFF, state >> 32


def displaced(first, second, d1, d2, count):
    return ((d2 + first * d1 + second) & 0xFFFFFFFF) % count


def perfect_hash(names):
    """Find the least seed whose hash-and-displace table maps each name to its own slot.

    Returns (seed, longest, displacements, slots) where slots[i] is the index in `names`
    of the name stored in slot i. The search is deterministic.
    """
    count = len(names)
    if count == 0 or len(set(names)) != count:
        raise ValueError("a perfect hash needs distinct names")
    units = [[ord(char) for char in name] for name in names]
    if any(ord(char) > 0x7F for name in names for char in name):
        raise ValueError("a vocabulary name that is not ASCII needs explicit handling: recognition compares units with bytes")
    buckets = max(1, -(-count // 4))
    for seed in range(1 << 16):
        parts = [hash_parts(hash_units(name, seed)) for name in units]
        grouped = [[] for _ in range(buckets)]
        for index, (selector, first, second) in enumerate(parts):
            grouped[selector % buckets].append(index)
        order = sorted(range(buckets), key=lambda bucket: (-len(grouped[bucket]), bucket))
        slots = [None] * count
        displacements = [(0, 0)] * buckets
        for bucket in order:
            members = grouped[bucket]
            if not members:
                continue
            found = None
            for d1 in range(count):
                for d2 in range(count):
                    targets = [displaced(parts[m][1], parts[m][2], d1, d2, count) for m in members]
                    if len(set(targets)) == len(targets) and all(slots[t] is None for t in targets):
                        found = (d1, d2, targets)
                        break
                if found:
                    break
            if found is None:
                break
            d1, d2, targets = found
            displacements[bucket] = (d1, d2)
            for member, target in zip(members, targets):
                slots[target] = member
        else:
            if any(slot is None for slot in slots):
                raise ValueError("perfect hash left a slot empty")
            longest = max(len(name) for name in units)
            for index, name in enumerate(units):
                selector, first, second = hash_parts(hash_units(name, seed))
                d1, d2 = displacements[selector % buckets]
                if slots[displaced(first, second, d1, d2, count)] != index:
                    raise ValueError("perfect hash lookup disagrees with its construction")
            return seed, longest, displacements, slots
    raise ValueError("no seed gives a perfect hash")


def hash_definition(constant, names):
    seed, longest, displacements, slots = perfect_hash(names)
    pairs = ", ".join(f"({d1}, {d2})" for d1, d2 in displacements)
    entries = ", ".join(str(slot) for slot in slots)
    return f"""#[rustfmt::skip]
const {constant}: NameHash = NameHash::new({seed}, {longest}, &[{pairs}], &[{entries}]);
"""


def unescape(text):
    return html.unescape(text)


FOREIGN_NAME = re.compile(r"\[(MathML|SVG) <code>([^<]+)</code>\]")


def foreign_names(text):
    """The (namespace, name) pairs of the linked foreign element names in `text`, in order."""
    return [(namespace, unescape(name)) for namespace, name in FOREIGN_NAME.findall(text)]


def list_after(lines, marker):
    """The bullet lines that follow the unique line containing `marker`."""
    found = [number for number, line in enumerate(lines) if marker in line]
    if len(found) != 1:
        raise ValueError(f"expected one line containing {marker!r}, found {len(found)}")
    start = found[0] + 1
    while start < len(lines) and not lines[start].strip():
        start += 1
    end = start
    while end < len(lines) and lines[end].startswith("- "):
        end += 1
    if end == start:
        raise ValueError(f"no list follows {marker!r}")
    return lines[start:end]


def generate_foreign(syntax_text):
    """The SVG tag adjustments and the foreign element names of the tree construction algorithm."""
    if f"Source revision `{REVISION}`" not in syntax_text:
        raise ValueError("retained syntax capture does not identify the selected HTML edition")
    stripped = re.sub(r'<a id="[^"]*"></a>', "", syntax_text)
    lines = stripped.splitlines()

    heading = "<th> Tag name </th><th> Element name      </th>"
    tables = [line for line in lines if heading in line]
    if len(tables) != 1:
        raise ValueError("expected one SVG tag name adjustment table")
    adjustments = [
        (unescape(original), unescape(adjusted))
        for original, adjusted in re.findall(r"<td> <code>([^<]+)</code> </td><td> <code>([^<]+)</code>", tables[0])
    ]
    if len(adjustments) != 37 or len({original for original, _ in adjustments}) != 37:
        raise ValueError("changed SVG tag name adjustment table")
    if any(original != adjusted.lower() for original, adjusted in adjustments):
        raise ValueError("an SVG tag name adjustment changes more than letter case")

    def only(namespace, pairs):
        if any(found != namespace for found, _ in pairs):
            raise ValueError(f"unexpected namespace in {pairs}")
        return [name for _, name in pairs]

    text_points = list_after(lines, "A node is a <strong>MathML text integration point</strong>")
    html_points = list_after(lines, "A node is an <strong>HTML integration point</strong>")
    scope = list_after(lines, "have a particular element in scope</strong>")
    special = [line for line in lines if line.startswith("The following elements have varying levels of special parsing rules")]
    script = [
        line for line in lines
        if line.startswith("<strong>An end tag whose tag name is &#34;script&#34;") and "[SVG <code>script</code>]" in line
    ]
    if len(special) != 1 or len(script) != 1:
        raise ValueError("changed special element paragraph or SVG script end tag rule")

    mathml_text = only("MathML", foreign_names("\n".join(text_points)))
    html_integration = foreign_names("\n".join(html_points))
    mathml_conditional = list(dict.fromkeys(only("MathML", [p for p in html_integration if p[0] == "MathML"])))
    svg_integration = only("SVG", [p for p in html_integration if p[0] == "SVG"])
    scope_pairs = foreign_names("\n".join(scope))
    special_pairs = foreign_names(special[0])
    svg_script = only("SVG", foreign_names(script[0]))
    if mathml_text != ["mi", "mo", "mn", "ms", "mtext"]:
        raise ValueError(f"changed MathML text integration points: {mathml_text}")
    if mathml_conditional != ["annotation-xml"] or svg_integration != ["foreignObject", "desc", "title"]:
        raise ValueError("changed HTML integration points")
    if svg_script != ["script"]:
        raise ValueError("changed SVG script element")
    mathml_special = only("MathML", [p for p in special_pairs if p[0] == "MathML"])
    svg_special = only("SVG", [p for p in special_pairs if p[0] == "SVG"])
    mathml_scope = only("MathML", [p for p in scope_pairs if p[0] == "MathML"])
    svg_scope = only("SVG", [p for p in scope_pairs if p[0] == "SVG"])
    if mathml_special != mathml_text + mathml_conditional or svg_special != svg_integration:
        raise ValueError("the special elements are no longer the integration points")
    if mathml_scope != mathml_special or svg_scope != svg_special:
        raise ValueError("the scope boundaries differ from the special foreign elements")

    mathml_names = mathml_special
    svg_names = [adjusted for _, adjusted in adjustments]
    for name in svg_special + svg_script:
        if name not in svg_names:
            svg_names.append(name)
    return {
        "adjustments": adjustments,
        "mathml": mathml_names,
        "svg": svg_names,
        "mathml_text": mathml_text,
        "svg_integration": svg_integration,
        "mathml_special": mathml_special,
        "svg_special": svg_special,
        "mathml_scope": mathml_scope,
        "svg_scope": svg_scope,
    }


def group_definition(constant, enum, names, description):
    values = ", ".join(f"{enum}::{variant(name)}" for name in names)
    return f"""/// {description}
pub const {constant}: [{enum}; {len(names)}] = [{values}];
"""


def foreign_definitions(foreign):
    output = "\n" + enum_definition(
        "KnownMathmlElement", "mathml", foreign["mathml"],
        "MathML element names that the pinned tree construction algorithm tests by name.", "MATHML_NAME_HASH")
    output += "\n" + enum_definition(
        "KnownSvgElement", "svg", foreign["svg"],
        "SVG element names that the pinned tree construction algorithm adjusts or tests by name.", "SVG_NAME_HASH")
    output += "\n" + hash_definition("MATHML_NAME_HASH", foreign["mathml"])
    output += hash_definition("SVG_NAME_HASH", foreign["svg"])
    pairs = ", ".join(
        f"({rust_string(original)}, KnownSvgElement::{variant(adjusted)})" for original, adjusted in foreign["adjustments"])
    output += f"""
/// The pinned SVG tag name adjustment table (rules for parsing tokens in foreign content,
/// 13.2.6.5): each lowercase tag name of an SVG element and the mixed-case element name
/// it becomes, in the table's order.
#[rustfmt::skip]
pub const SVG_TAG_NAME_ADJUSTMENTS: &[(&str, KnownSvgElement)] = &[{pairs}];

"""
    output += "#[rustfmt::skip]\n" + group_definition(
        "MATHML_TEXT_INTEGRATION_POINTS", "KnownMathmlElement", foreign["mathml_text"],
        "The MathML elements that are MathML text integration points (13.2.6).")
    output += "\n#[rustfmt::skip]\n" + group_definition(
        "SVG_HTML_INTEGRATION_POINTS", "KnownSvgElement", foreign["svg_integration"],
        "The SVG elements that are HTML integration points by name (13.2.6). A MathML `annotation-xml`\n/// is one only with a matching `encoding` attribute.")
    output += "\n#[rustfmt::skip]\n" + group_definition(
        "MATHML_SPECIAL_ELEMENTS", "KnownMathmlElement", foreign["mathml_special"],
        "The MathML elements in the special category (13.2.4.2).")
    output += "\n#[rustfmt::skip]\n" + group_definition(
        "SVG_SPECIAL_ELEMENTS", "KnownSvgElement", foreign["svg_special"],
        "The SVG elements in the special category (13.2.4.2).")
    output += "\n#[rustfmt::skip]\n" + group_definition(
        "MATHML_SCOPE_BOUNDARIES", "KnownMathmlElement", foreign["mathml_scope"],
        "The MathML elements among the element types of the default scope (13.2.4.2).")
    output += "\n#[rustfmt::skip]\n" + group_definition(
        "SVG_SCOPE_BOUNDARIES", "KnownSvgElement", foreign["svg_scope"],
        "The SVG elements among the element types of the default scope (13.2.4.2).")
    return output


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
    handlers = table(parser, "List of event handler content attributes", 4, 89)

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

    handler_names = [row[0]["text"] for row in handlers]
    handler_scopes = [row[1]["text"] for row in handlers]
    if len(set(handler_names)) != len(handler_names) or any(not re.fullmatch(r"on[a-z]+", name) for name in handler_names):
        raise ValueError("event handler content attribute names require explicit handling")
    if set(handler_scopes) != {"HTML elements", "body"}:
        raise ValueError("event handler applicability requires explicit handling")
    syntax_raw = (ROOT / SYNTAX_CAPTURE).read_bytes()
    if hashlib.sha256(syntax_raw).hexdigest() != SYNTAX_CAPTURE_SHA256:
        raise ValueError("retained syntax capture is not the pinned input")
    foreign = generate_foreign(syntax_raw.decode("utf-8"))
    output = "// Generated by tools/generate_vocabulary.py from retained non-normative indexes and\n"
    output += "// the retained syntax capture of the same edition.\n\n"
    output += enum_definition("KnownHtmlElement", "html", element_names,
                              "Individually named HTML elements in the pinned modern element index.",
                              "ELEMENT_NAME_HASH")
    output += "\n" + enum_definition("KnownHtmlAttribute", "none", attribute_names,
                                    "Local names in the pinned ordinary attribute index; contextual rows remain separate.")
    output += "\n" + enum_definition("KnownHtmlEventHandler", "none", handler_names,
                                    "Event handler content attributes in the pinned event handler index.",
                                    "EVENT_HANDLER_NAME_HASH")
    output += "\n" + hash_definition("ELEMENT_NAME_HASH", element_names)
    output += hash_definition("EVENT_HANDLER_NAME_HASH", handler_names)
    output += foreign_definitions(foreign)
    output += "\nimpl KnownHtmlEventHandler {\n"
    output += "    /// The elements this event handler content attribute applies to, as the index states.\n"
    output += "    #[rustfmt::skip]\n    pub fn applicability(self) -> HtmlEventHandlerApplicability {\n        match self {\n"
    for name, scope in zip(handler_names, handler_scopes):
        target = "AllHtmlElements" if scope == "HTML elements" else "BodyElement"
        output += f"            Self::{variant(name)} => HtmlEventHandlerApplicability::{target},\n"
    output += "        }\n    }\n}\n"
    output += f'''
/// Immutable source identity for these generated inventory records.
pub const VOCABULARY_SOURCE: VocabularySource = VocabularySource {{
    revision: {rust_string(REVISION)},
    original_html_sha256: {rust_string(ORIGINAL_SHA256)},
    retained_capture: {rust_string(CAPTURE)},
    retained_capture_sha256: {rust_string(hashlib.sha256(raw).hexdigest())},
    retained_syntax_capture: {rust_string(SYNTAX_CAPTURE)},
    retained_syntax_capture_sha256: {rust_string(hashlib.sha256(syntax_raw).hexdigest())},
    elements_url: {rust_string(SNAPSHOT + "#elements-3")},
    attributes_url: {rust_string(SNAPSHOT + "#attributes-3")},
}};

#[rustfmt::skip]
static ELEMENT_INVENTORY: [ElementInventoryRow; {len(elements)}] = [
'''
    for number, (row, names) in enumerate(zip(elements, row_names), 1):
        values = [cell["text"] for cell in row]
        output += "    ElementInventoryRow {\n"
        output += f"        row: {number},\n        name: {rust_string(values[0])},\n"
        output += "        html_elements: " + array(f"KnownHtmlElement::{variant(name)}" for name in names) + ",\n"
        for field, value in zip(("description", "categories", "parents", "children", "attributes", "interface"), values[1:]):
            output += f"        {field}: {rust_string(value)},\n"
        output += "        references: " + array(rust_string(ref) for ref in references(row[:1])) + ",\n    },\n"
    output += "];\n\n"
    # Each element maps to the first row that mentions it, by a literal index into the
    # fixed-size array, so that the compiler rejects an index outside the table.
    first_row = {}
    for index, names in enumerate(row_names):
        for name in names:
            first_row.setdefault(name, index)
    output += """impl KnownHtmlElement {
    /// Returns the source index row mentioning this element.
    ///
    /// The heading elements share their single original index row.
    #[rustfmt::skip]
    pub fn inventory_row(self) -> &'static ElementInventoryRow {
        match self {
"""
    for name in element_names:
        output += f"            Self::{variant(name)} => &ELEMENT_INVENTORY[{first_row[name]}],\n"
    output += "        }\n    }\n}\n\n"
    output += "#[rustfmt::skip]\nstatic ATTRIBUTE_INVENTORY: &[AttributeInventoryRow] = &[\n"
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
        print("HTML vocabulary inventory matches retained source (111 rows / 113 HTML names; 180 rows / 144 attribute names; 89 event handler names; 37 SVG tag adjustments; 6 MathML and 40 SVG foreign names)")
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
